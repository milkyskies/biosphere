use bevy::{prelude::*, utils::info};
use noise::{NoiseFn, Perlin};

const GRID_WIDTH: usize = 32;
const GRID_HEIGHT: usize = 32;
const CELL_SIZE: f32 = 64.0;
const INITIAL_TEMPERATURE: f32 = 50.0;
const TILE_MASS: f32 = 0.01;
const TILE_HEAT_CAPACITY: f32 = 1.0;
const HEAT_TRANSFER_RATE: f32 = 0.001; // Adjust this value to control the rate
const MINIMUM_HEAT: f32 = 0.0;
const MAXIMUM_HEAT: f32 = 100.0;

pub struct HeatDiffusionPlugin;

impl Plugin for HeatDiffusionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
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

fn setup(mut commands: Commands) {
    let perlin = Perlin::new(rand::random::<u32>());
    let scale = 0.1; // Adjust the scale to control the noise frequency

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

fn heat_diffusion(mut query: Query<(&GridPosition, &mut Temperature)>, time: Res<Time>) {
    let mut new_temperatures = vec![vec![INITIAL_TEMPERATURE; GRID_HEIGHT]; GRID_WIDTH];
    let mut heat_flux_grid = vec![vec![0.0; GRID_HEIGHT]; GRID_WIDTH];

    for (grid_position, temperature) in query.iter() {
        for (dx, dy) in [(1, 0), (0, 1)].iter() {
            let neighbor_x = grid_position.x as isize + dx;
            let neighbor_y = grid_position.y as isize + dy;

            if let Some((_, neighbor_temp)) = query.iter().find(|(neighbor_pos, _)| {
                neighbor_pos.x == neighbor_x as usize && neighbor_pos.y == neighbor_y as usize
            }) {
                let flux = calculate_heat_flux(temperature.0, neighbor_temp.0);

                heat_flux_grid[grid_position.x][grid_position.y] -= flux;
                heat_flux_grid[neighbor_x as usize][neighbor_y as usize] += flux;
            }
        }
    }

    for (pos, temp) in query.iter() {
        let heat_flux = heat_flux_grid[pos.x][pos.y];
        let new_temp =
            temp.0 + (heat_flux / (TILE_MASS * TILE_HEAT_CAPACITY)) * 0.001 * time.delta_seconds();

        new_temperatures[pos.x][pos.y] = new_temp.clamp(MINIMUM_HEAT, MAXIMUM_HEAT);
    }

    for (pos, mut temperature) in query.iter_mut() {
        temperature.0 = new_temperatures[pos.x][pos.y];
    }
}

fn visualize_temperature(mut query: Query<(&Temperature, &mut Sprite)>) {
    for (temp, mut sprite) in query.iter_mut() {
        let temperature_ratio = (temp.0) / (100.0);

        sprite.color = Color::rgb(temperature_ratio, 0.0, 1.0 - temperature_ratio);

        // text.sections.clear();

        // text.sections.push(TextSection::new(
        //     format!("{:.1}", temp.0),
        //     TextStyle {
        //         color: Color::rgb(1.0, 1.0, 1.0),
        //         ..Default::default()
        //     },
        // ));
    }
}

// fn visualize_temperature(mut query: Query<(&Temperature, &mut Text)>) {
//     for (temp, mut text) in query.iter_mut() {
//         // let temperature_ratio = (temp.0 - ABSOLUTE_ZERO) / (MAXIMUM_HEAT - ABSOLUTE_ZERO);

//         // sprite.color = Color::rgb(temperature_ratio, 0.0, 1.0 - temperature_ratio);
//         text.sections.clear();

//         text.sections.push(TextSection::new(
//             format!("{:.1}", temp.0),
//             TextStyle {
//                 color: Color::rgb(0.2, 0.2, 0.2),
//                 ..Default::default()
//             },
//         ));
//     }
// }
