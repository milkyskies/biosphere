use bevy::prelude::*;
use noise::{NoiseFn, Perlin};

const INITIAL_TEMPERATURE: f32 = 50.0;
const TILE_MASS: f32 = 0.5;
const HEAT_TRANSFER_SPEED: f32 = 1.0;
const TILE_HEAT_CAPACITY: f32 = 1.0;
const MINIMUM_HEAT: f32 = 0.0;
const MAXIMUM_HEAT: f32 = 100.0;
const CHUNK_SIZE: usize = 16; // Make sure this is divisible by grid_width and grid_height
const CHUNK_CONSTANT: usize = 256;
const CHUNKING_FACTOR: f32 = (CHUNK_CONSTANT as f32 / (CHUNK_SIZE.pow(2) as f32)) as f32;

pub struct HeatDiffusionPlugin {
    pub world_size: Vec2,
    pub grid_width: usize,
    pub grid_height: usize,
    pub cell_size: f32,
}

impl Plugin for HeatDiffusionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(HeatDiffusionConfig {
            grid_width: self.grid_width,
            grid_height: self.grid_height,
            cell_size: self.cell_size,
            world_size: self.world_size,
        })
        .insert_resource(CurrentChunk { x: 0, y: 0 })
        .insert_resource(HeatFluxGrid {
            grid: vec![vec![0.0; self.grid_height]; self.grid_width],
        })
        .insert_resource(ProcessedTileCount(0))
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (calculate_heat_diffusion, apply_heat_diffusion).chain(),
        )
        .add_systems(Update, visualize_temperature);
    }
}

#[derive(Resource)]
struct HeatDiffusionConfig {
    grid_width: usize,
    grid_height: usize,
    cell_size: f32,
    world_size: Vec2,
}

#[derive(Component)]
struct Temperature(f32);

#[derive(Component)]
struct GridPosition {
    x: usize,
    y: usize,
}

#[derive(Resource)]
struct CurrentChunk {
    x: usize,
    y: usize,
}

#[derive(Resource)]
struct HeatFluxGrid {
    grid: Vec<Vec<f32>>,
}

#[derive(Resource)]
struct ProcessedTileCount(usize);

fn setup(mut commands: Commands, config: Res<HeatDiffusionConfig>) {
    let perlin = Perlin::new(rand::random::<u32>());
    let scale = 0.1;

    let offset_x = (config.world_size.x - config.grid_width as f32 * config.cell_size) / 2.0;
    let offset_y = (config.world_size.y - config.grid_height as f32 * config.cell_size) / 2.0;

    for x in 0..config.grid_width {
        for y in 0..config.grid_height {
            let noise_value = perlin.get([x as f64 * scale, y as f64 * scale]);
            let temperature = ((noise_value + 1.0) / 2.0) * 100.0; // Normalize to [0, 100]

            commands.spawn((
                GridPosition { x, y },
                Temperature(temperature as f32),
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(config.cell_size, config.cell_size)),
                        ..Default::default()
                    },
                    transform: Transform::from_translation(Vec3::new(
                        x as f32 * config.cell_size + offset_x - config.world_size.x / 2.0,
                        y as f32 * config.cell_size + offset_y - config.world_size.y / 2.0,
                        0.0,
                    )),
                    ..Default::default()
                },
            ));
        }
    }
}

fn calculate_heat_flux(temp1: f32, temp2: f32) -> f32 {
    let temp_mid = (temp1 + temp2) / 2.0;
    let thermal_conductivity = 0.6065 - 0.00122 * temp_mid + 0.0000063 * temp_mid.powi(2);

    thermal_conductivity * (temp1 - temp2)
}

fn calculate_heat_diffusion(
    query: Query<(&GridPosition, &mut Temperature)>,
    mut current_chunk: ResMut<CurrentChunk>,
    mut heat_flux_grid: ResMut<HeatFluxGrid>,
    mut processed_tile_count: ResMut<ProcessedTileCount>,
    config: Res<HeatDiffusionConfig>,
) {
    let mut temperature_grid = vec![vec![0.0; config.grid_height]; config.grid_width];
    for (pos, temperature) in query.iter() {
        temperature_grid[pos.x][pos.y] = temperature.0;
    }

    // Calculate the starting and ending indices for the current chunk
    let start_x = current_chunk.x * CHUNK_SIZE;
    let start_y = current_chunk.y * CHUNK_SIZE;
    let end_x = (start_x + CHUNK_SIZE).min(config.grid_width);
    let end_y = (start_y + CHUNK_SIZE).min(config.grid_height);

    // Iterate over each cell in the chunk
    for x in start_x..end_x {
        for y in start_y..end_y {
            let current_temp = temperature_grid[x][y];

            // Check the neighboring cells in the east and south directions
            for (dx, dy) in [(1, 0), (0, 1)].iter() {
                let neighbor_x = x as isize + dx;
                let neighbor_y = y as isize + dy;

                if neighbor_x >= 0
                    && neighbor_x < config.grid_width as isize
                    && neighbor_y >= 0
                    && neighbor_y < config.grid_height as isize
                {
                    let neighbor_temp = temperature_grid[neighbor_x as usize][neighbor_y as usize];

                    // Calculate the heat flux between the current cell and its neighbor
                    let flux = calculate_heat_flux(current_temp, neighbor_temp) * CHUNKING_FACTOR;

                    // Update the heat flux grid resource for both the current cell and the neighbor
                    heat_flux_grid.grid[x][y] -= flux;
                    heat_flux_grid.grid[neighbor_x as usize][neighbor_y as usize] += flux;
                }
            }

            processed_tile_count.0 += 1;
        }
    }

    // Move to the next chunk, wrapping around if necessary
    current_chunk.x += 1;
    if current_chunk.x * CHUNK_SIZE >= config.grid_width {
        current_chunk.x = 0;
        current_chunk.y += 1;
        if current_chunk.y * CHUNK_SIZE >= config.grid_height {
            current_chunk.y = 0;
        }
    }
}

fn apply_heat_diffusion(
    mut query: Query<(&GridPosition, &mut Temperature)>,
    time: Res<Time>,
    mut heat_flux_grid: ResMut<HeatFluxGrid>,
    mut processed_tile_count: ResMut<ProcessedTileCount>,
    config: Res<HeatDiffusionConfig>,
) {
    // Only apply heat diffusion if we are processing the first chunk (meaning that a full cycle has been completed)
    if processed_tile_count.0 != config.grid_width * config.grid_height {
        return;
    }

    // Prepare a grid to store the new temperatures
    let mut new_temperatures: Vec<Vec<f32>> =
        vec![vec![INITIAL_TEMPERATURE; config.grid_height]; config.grid_width];

    // Calculate the new temperature for each cell based on the heat flux
    for (pos, temp) in query.iter() {
        let heat_flux = heat_flux_grid.grid[pos.x][pos.y];
        let new_temp = temp.0
            + (heat_flux / (TILE_MASS * TILE_HEAT_CAPACITY))
                * HEAT_TRANSFER_SPEED
                * time.delta_seconds();

        // Clamp the new temperature to the valid range and store it
        new_temperatures[pos.x][pos.y] = new_temp.clamp(MINIMUM_HEAT, MAXIMUM_HEAT);
    }

    // Update the actual temperatures of the grid tiles with the new temperatures
    for (pos, mut temperature) in query.iter_mut() {
        temperature.0 = new_temperatures[pos.x][pos.y];
    }

    // Clear the heat flux grid
    heat_flux_grid.grid = vec![vec![0.0; config.grid_height]; config.grid_width];

    processed_tile_count.0 = 0;
}

fn visualize_temperature(mut query: Query<(&Temperature, &mut Sprite)>) {
    for (temp, mut sprite) in query.iter_mut() {
        let temperature_ratio = (temp.0) / (100.0);

        sprite.color = Color::srgb(temperature_ratio, 0.0, 1.0 - temperature_ratio);
    }
}
