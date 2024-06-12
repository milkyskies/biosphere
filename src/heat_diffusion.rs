use crate::WORLD_SIZE;
use bevy::prelude::*;
use noise::{NoiseFn, Perlin};

const GRID_WIDTH: usize = 32;
const GRID_HEIGHT: usize = 32;
const CELL_SIZE: f32 = 32.0;
const INITIAL_TEMPERATURE: f32 = 50.0;
const TILE_MASS: f32 = 0.05;
const HEAT_TRANSFER_SPEED: f32 = 1.0;
const TILE_HEAT_CAPACITY: f32 = 1.0;
const MINIMUM_HEAT: f32 = 0.0;
const MAXIMUM_HEAT: f32 = 100.0;
const CHUNK_SIZE: usize = 16;
const DEBUG_CHUNK_SIZE: usize = 8;

pub struct HeatDiffusionPlugin;

impl Plugin for HeatDiffusionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentChunk { x: 0, y: 0 })
            .insert_resource(DebugCurrentChunk { x: 0, y: 0 })
            .insert_resource(HeatFluxGrid {
                grid: vec![vec![0.0; GRID_HEIGHT]; GRID_WIDTH],
            })
            .insert_resource(DebugHeatFluxGrid {
                grid: vec![vec![0.0; GRID_HEIGHT]; GRID_WIDTH],
            })
            .add_systems(Startup, setup)
            .add_systems(
                FixedUpdate,
                (calculate_heat_diffusion, apply_heat_diffusion).chain(),
            )
            .add_systems(
                FixedUpdate,
                (debug_calculate_heat_diffusion, debug_apply_heat_diffusion).chain(),
            )
            .add_systems(Update, visualize_temperature);
    }
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
struct DebugCurrentChunk {
    x: usize,
    y: usize,
}

#[derive(Component)]
struct DebugGuy;

#[derive(Resource)]

struct HeatFluxGrid {
    grid: Vec<Vec<f32>>,
}

#[derive(Resource)]

struct DebugHeatFluxGrid {
    grid: Vec<Vec<f32>>,
}

fn setup(mut commands: Commands) {
    let perlin = Perlin::new(rand::random::<u32>());
    let scale = 0.1;

    let offset_x = (WORLD_SIZE.x - GRID_WIDTH as f32 * CELL_SIZE) / 2.0;
    let offset_y = (WORLD_SIZE.y - GRID_HEIGHT as f32 * CELL_SIZE) / 2.0;

    for x in 0..GRID_WIDTH {
        for y in 0..GRID_HEIGHT {
            let noise_value = perlin.get([x as f64 * scale, y as f64 * scale]);
            let temperature = ((noise_value + 1.0) / 2.0) * 100.0; // Normalize to [0, 100]

            commands.spawn((
                GridPosition { x, y },
                Temperature(temperature as f32),
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(CELL_SIZE, CELL_SIZE)),
                        ..Default::default()
                    },
                    transform: Transform::from_translation(Vec3::new(
                        x as f32 * CELL_SIZE + offset_x - WORLD_SIZE.x / 2.0,
                        y as f32 * CELL_SIZE + offset_y - WORLD_SIZE.y / 2.0,
                        -1.0,
                    )),
                    ..Default::default()
                },
            ));
        }
    }

    let x_shift = GRID_WIDTH as f32 * CELL_SIZE * 1.5;

    for x in 0..GRID_WIDTH {
        for y in 0..GRID_HEIGHT {
            let noise_value = perlin.get([x as f64 * scale, y as f64 * scale]);
            let temperature = ((noise_value + 1.0) / 2.0) * 100.0; // Normalize to [0, 100]

            commands.spawn((
                GridPosition { x, y },
                Temperature(temperature as f32),
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(CELL_SIZE, CELL_SIZE)),
                        ..Default::default()
                    },
                    transform: Transform::from_translation(Vec3::new(
                        x as f32 * CELL_SIZE + offset_x - WORLD_SIZE.x / 2.0 + x_shift,
                        y as f32 * CELL_SIZE + offset_y - WORLD_SIZE.y / 2.0,
                        -1.0,
                    )),
                    ..Default::default()
                },
                DebugGuy,
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
    query: Query<(&GridPosition, &mut Temperature), Without<DebugGuy>>,
    mut current_chunk: ResMut<CurrentChunk>,
    mut heat_flux_grid: ResMut<HeatFluxGrid>,
) {
    // Calculate the starting and ending indices for the current chunk
    let start_x = current_chunk.x * CHUNK_SIZE;
    let start_y = current_chunk.y * CHUNK_SIZE;
    let end_x = (start_x + CHUNK_SIZE).min(GRID_WIDTH);
    let end_y = (start_y + CHUNK_SIZE).min(GRID_HEIGHT);

    // Iterate over each cell in the chunk
    for x in start_x..end_x {
        for y in start_y..end_y {
            // Find the temperature component of the current cell
            if let Some((_, temperature)) = query.iter().find(|(pos, _)| pos.x == x && pos.y == y) {
                // Check the neighboring cells in the east and south directions
                for (dx, dy) in [(1, 0), (0, 1)].iter() {
                    let neighbor_x = x as isize + dx;
                    let neighbor_y = y as isize + dy;

                    // Find the temperature component of the neighboring cell
                    if let Some((_, neighbor_temp)) = query.iter().find(|(neighbor_pos, _)| {
                        neighbor_pos.x == neighbor_x as usize
                            && neighbor_pos.y == neighbor_y as usize
                    }) {
                        // Calculate the heat flux between the current cell and its neighbor
                        let flux = calculate_heat_flux(temperature.0, neighbor_temp.0);

                        // Update the heat flux grid resource for both the current cell and the neighbor
                        heat_flux_grid.grid[x][y] -= flux;
                        heat_flux_grid.grid[neighbor_x as usize][neighbor_y as usize] += flux;
                    }
                }
            }
        }
    }

    // Move to the next chunk, wrapping around if necessary
    current_chunk.x += 1;
    if current_chunk.x * CHUNK_SIZE >= GRID_WIDTH {
        current_chunk.x = 0;
        current_chunk.y += 1;
        if current_chunk.y * CHUNK_SIZE >= GRID_HEIGHT {
            current_chunk.y = 0;
        }
    }
}

fn apply_heat_diffusion(
    mut query: Query<(&GridPosition, &mut Temperature), Without<DebugGuy>>,
    time: Res<Time>,
    mut heat_flux_grid: ResMut<HeatFluxGrid>,
    current_chunk: Res<CurrentChunk>,
) {
    // Only apply heat diffusion if we are processing the first chunk (meaning that a full cycle has been completed)
    if current_chunk.x != 0 || current_chunk.y != 0 {
        return;
    }

    // Prepare a grid to store the new temperatures
    let mut new_temperatures: Vec<Vec<f32>> =
        vec![vec![INITIAL_TEMPERATURE; GRID_HEIGHT]; GRID_WIDTH];

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

    // Reset the heat flux grid after updating the temperatures
    reset_heat_flux_grid(&mut heat_flux_grid);
}

fn debug_calculate_heat_diffusion(
    query: Query<(&GridPosition, &mut Temperature), With<DebugGuy>>,
    mut current_chunk: ResMut<DebugCurrentChunk>,
    mut heat_flux_grid: ResMut<DebugHeatFluxGrid>,
) {
    let start_x = current_chunk.x * DEBUG_CHUNK_SIZE;
    let start_y = current_chunk.y * DEBUG_CHUNK_SIZE;
    let end_x = (start_x + DEBUG_CHUNK_SIZE).min(GRID_WIDTH);
    let end_y = (start_y + DEBUG_CHUNK_SIZE).min(GRID_HEIGHT);

    for x in start_x..end_x {
        for y in start_y..end_y {
            if let Some((_, temperature)) = query.iter().find(|(pos, _)| pos.x == x && pos.y == y) {
                for (dx, dy) in [(1, 0), (0, 1)].iter() {
                    let neighbor_x = x as isize + dx;
                    let neighbor_y = y as isize + dy;

                    if let Some((_, neighbor_temp)) = query.iter().find(|(neighbor_pos, _)| {
                        neighbor_pos.x == neighbor_x as usize
                            && neighbor_pos.y == neighbor_y as usize
                    }) {
                        let flux = calculate_heat_flux(temperature.0, neighbor_temp.0);

                        heat_flux_grid.grid[x][y] -= flux;
                        heat_flux_grid.grid[neighbor_x as usize][neighbor_y as usize] += flux;
                    }
                }
            }
        }
    }

    current_chunk.x += 1;

    if current_chunk.x * DEBUG_CHUNK_SIZE >= GRID_WIDTH {
        current_chunk.x = 0;
        current_chunk.y += 1;

        if current_chunk.y * DEBUG_CHUNK_SIZE >= GRID_HEIGHT {
            current_chunk.y = 0;
        }
    }
}

fn debug_apply_heat_diffusion(
    mut query: Query<(&GridPosition, &mut Temperature), With<DebugGuy>>,
    time: Res<Time>,
    mut heat_flux_grid: ResMut<DebugHeatFluxGrid>,
    current_chunk: Res<DebugCurrentChunk>,
) {
    if current_chunk.x != 0 || current_chunk.y != 0 {
        return;
    }

    let mut new_temperatures: Vec<Vec<f32>> =
        vec![vec![INITIAL_TEMPERATURE; GRID_HEIGHT]; GRID_WIDTH];

    for (pos, temp) in query.iter() {
        let heat_flux = heat_flux_grid.grid[pos.x][pos.y];
        let new_temp = temp.0
            + (heat_flux / (TILE_MASS * TILE_HEAT_CAPACITY))
                * HEAT_TRANSFER_SPEED
                * time.delta_seconds();

        new_temperatures[pos.x][pos.y] = new_temp.clamp(MINIMUM_HEAT, MAXIMUM_HEAT);
    }

    for (pos, mut temperature) in query.iter_mut() {
        temperature.0 = new_temperatures[pos.x][pos.y];
    }

    reset_heat_flux_grid_debug(&mut heat_flux_grid);
}

fn reset_heat_flux_grid(heat_flux_grid: &mut HeatFluxGrid) {
    for row in &mut heat_flux_grid.grid {
        for cell in row {
            *cell = 0.0;
        }
    }
}

fn reset_heat_flux_grid_debug(heat_flux_grid: &mut DebugHeatFluxGrid) {
    for row in &mut heat_flux_grid.grid {
        for cell in row {
            *cell = 0.0;
        }
    }
}

fn visualize_temperature(mut query: Query<(&Temperature, &mut Sprite)>) {
    for (temp, mut sprite) in query.iter_mut() {
        let temperature_ratio = (temp.0) / (100.0);

        sprite.color = Color::rgb(temperature_ratio, 0.0, 1.0 - temperature_ratio);
    }
}
