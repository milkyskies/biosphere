use bevy::{
    input::mouse::MouseWheel, prelude::*, sprite::MaterialMesh2dBundle, window::PrimaryWindow,
};

mod camera;
mod stepping;

const ORGANISM_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
const WORLD_SIZE: Vec2 = Vec2::new(1024.0, 1024.0);

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
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (apply_velocity)
                // `chain`ing systems together runs them in order
                .chain(),
        )
        .add_systems(Update, (bevy::window::close_on_esc))
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
    let organism_material_handle = materials.add(ORGANISM_COLOR);

    (0..500000).for_each(|_| {
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: organism_mesh_handle.clone().into(),
                material: organism_material_handle.clone(),
                transform: Transform::from_translation(Vec3::new(
                    rand::random::<f32>() * WORLD_SIZE.x - WORLD_SIZE.x / 2.0,
                    rand::random::<f32>() * WORLD_SIZE.y - WORLD_SIZE.y / 2.0,
                    0.0,
                ))
                .with_scale(Vec3::new(
                    rand::random::<f32>() * 0.5 + 0.5,
                    rand::random::<f32>() * 0.5 + 0.5,
                    1.0,
                )),
                ..default()
            },
            Organism,
            Velocity(Vec2::new(
                rand::random::<f32>() * 2.0 - 1.0,
                rand::random::<f32>() * 2.0 - 1.0,
            )),
        ));
    });
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * time.delta_seconds();
        transform.translation.y += velocity.y * time.delta_seconds();
    }
}
