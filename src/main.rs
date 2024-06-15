use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

mod camera;
mod heat_diffusion;
mod stepping;

const ORGANISM_COLOR: Color = Color::srgba(0.2, 0.8, 0.5, 0.6);
const GRID_WIDTH: usize = 64;
const GRID_HEIGHT: usize = 64;
const CELL_SIZE: f32 = 32.0;

const WORLD_SIZE: Vec2 = Vec2::new(
    GRID_WIDTH as f32 * CELL_SIZE,
    GRID_HEIGHT as f32 * CELL_SIZE,
);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(
            stepping::SteppingPlugin::default()
                .add_schedule(Update)
                .add_schedule(FixedUpdate)
                .at(Val::Percent(35.0), Val::Percent(50.0)),
        )
        .add_plugins(camera::CameraPlugin)
        .add_plugins(heat_diffusion::HeatDiffusionPlugin {
            grid_width: GRID_WIDTH,
            grid_height: GRID_HEIGHT,
            cell_size: CELL_SIZE,
            world_size: WORLD_SIZE,
        })
        .insert_resource(ClearColor(Color::srgb(0.8, 0.78, 0.88)))
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (apply_velocity)
                // `chain`ing systems together runs them in order
                .chain(),
        )
        .add_systems(Update, (bevy::window::close_when_requested))
        .run();
}

#[derive(Component)]
struct Organism;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let organism_mesh_handle = meshes.add(Circle::default());

    (0..5000).for_each(|_| {
        let position = Vec3::new(
            rand::random::<f32>() * WORLD_SIZE.x - WORLD_SIZE.x / 2.0,
            rand::random::<f32>() * WORLD_SIZE.y - WORLD_SIZE.y / 2.0,
            1.0,
        );

        let scale = Vec3::new(
            rand::random::<f32>() * 4.0 + 4.0,
            rand::random::<f32>() * 4.0 + 4.0,
            1.0,
        );

        let velocity = Vec2::new(
            rand::random::<f32>() * 16.0 - 8.0,
            rand::random::<f32>() * 16.0 - 8.0,
        );

        let linear_color: LinearRgba = ORGANISM_COLOR.into();

        let color_variation = Color::srgba(
            (linear_color.red + rand::random::<f32>() * 0.2 - 0.1).clamp(0.0, 1.0),
            (linear_color.green + rand::random::<f32>() * 0.2 - 0.1).clamp(0.0, 1.0),
            (linear_color.blue + rand::random::<f32>() * 0.2 - 0.1).clamp(0.0, 1.0),
            linear_color.alpha,
        );
        let organism_material_handle = materials.add(color_variation);

        commands.spawn((
            MaterialMesh2dBundle {
                mesh: organism_mesh_handle.clone().into(),
                material: organism_material_handle.clone(),
                transform: Transform::from_translation(position).with_scale(scale),
                ..default()
            },
            Organism,
            Velocity(velocity),
        ));
    });
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * time.delta_seconds();
        transform.translation.y += velocity.y * time.delta_seconds();
    }
}
