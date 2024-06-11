use bevy::{prelude::*, utils::info};
use noise::{NoiseFn, Perlin};

const GRID_WIDTH: usize = 32;
const GRID_HEIGHT: usize = 32;
const CELL_SIZE: f32 = 16.0;
const INITIAL_TEMPERATURE: f32 = 50.0;
const TILE_MASS: f32 = 0.5;
const TILE_HEAT_CAPACITY: f32 = 1.0;
const MINIMUM_HEAT: f32 = 0.0;
const MAXIMUM_HEAT: f32 = 100.0;
const CHUNK_SIZE: usize = 8;

pub struct HeatDiffusionPlugin;

impl Plugin for HeatDiffusionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentChunk { x: 0, y: 0 })
            .add_systems(Startup, setup)
            .add_systems(FixedUpdate, (heat_diffusion, visualize_temperature));
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

fn setup(mut commands: Commands) {
    let perlin = Perlin::new(rand::random::<u32>());
    let scale = 0.01;

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
                        x as f32 * CELL_SIZE,
                        y as f32 * CELL_SIZE,
                        -1.0,
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

fn heat_diffusion(
    mut query: Query<(&GridPosition, &mut Temperature)>,
    time: Res<Time>,
    mut current_chunk: ResMut<CurrentChunk>,
) {
    let mut new_temperatures = vec![vec![INITIAL_TEMPERATURE; GRID_HEIGHT]; GRID_WIDTH];
    let mut heat_flux_grid = vec![vec![0.0; GRID_HEIGHT]; GRID_WIDTH];

    let start_x = current_chunk.x * CHUNK_SIZE;
    let start_y = current_chunk.y * CHUNK_SIZE;
    let end_x = (start_x + CHUNK_SIZE).min(GRID_WIDTH);
    let end_y = (start_y + CHUNK_SIZE).min(GRID_HEIGHT);

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

                        heat_flux_grid[x][y] -= flux;
                        heat_flux_grid[neighbor_x as usize][neighbor_y as usize] += flux;
                    }
                }
            }
        }
    }

    for x in start_x..end_x {
        for y in start_y..end_y {
            if let Some((_, temp)) = query.iter().find(|(pos, _)| pos.x == x && pos.y == y) {
                let heat_flux = heat_flux_grid[x][y];
                let new_temp =
                    temp.0 + (heat_flux / (TILE_MASS * TILE_HEAT_CAPACITY)) * time.delta_seconds();
                new_temperatures[x][y] = new_temp.clamp(MINIMUM_HEAT, MAXIMUM_HEAT);
            }
        }
    }

    for x in start_x..end_x {
        for y in start_y..end_y {
            if let Some((_, mut temperature)) =
                query.iter_mut().find(|(pos, _)| pos.x == x && pos.y == y)
            {
                temperature.0 = new_temperatures[x][y];
            }
        }
    }

    current_chunk.x += 1;

    if current_chunk.x * CHUNK_SIZE >= GRID_WIDTH {
        current_chunk.x = 0;
        current_chunk.y += 1;

        if current_chunk.y * CHUNK_SIZE >= GRID_HEIGHT {
            current_chunk.y = 0;
        }
    }
}

fn visualize_temperature(mut query: Query<(&Temperature, &mut Sprite)>) {
    for (temp, mut sprite) in query.iter_mut() {
        let temperature_ratio = (temp.0) / (100.0);

        sprite.color = Color::rgb(temperature_ratio, 0.0, 1.0 - temperature_ratio);
    }
}
