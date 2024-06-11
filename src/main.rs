use bevy::{
    input::mouse::MouseWheel, math::primitives::Sphere, prelude::*, render::mesh::shape,
    sprite::MaterialMesh2dBundle, window::PrimaryWindow,
};

mod camera;
mod stepping;

const ORGANISM_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
// const WORLD_SIZE: Vec2 = Vec2::new(1024.0, 1024.0);
const WORLD_RADIUS: f32 = 100.0; // Half of your original WORLD_SIZE for a radius

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
struct Velocity(Vec3);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let organism_mesh_handle = meshes.add(Mesh::from(Sphere { radius: 0.5 }));
    let organism_material_handle = materials.add(StandardMaterial {
        base_color: ORGANISM_COLOR,
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::WHITE,
            illuminance: 10000.0, // Adjust the illuminance as needed
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)),
        ..default()
    });

    (0..5000).for_each(|_| {
        let rand_direction = Vec3::new(
            rand::random::<f32>() * 2.0 - 1.0,
            rand::random::<f32>() * 2.0 - 1.0,
            rand::random::<f32>() * 2.0 - 1.0,
        )
        .normalize();
        let rand_distance = rand::random::<f32>().sqrt() * WORLD_RADIUS; // sqrt for uniform distribution
        let position = rand_direction * rand_distance;

        commands
            .spawn(PbrBundle {
                mesh: organism_mesh_handle.clone(),
                material: organism_material_handle.clone(),
                transform: Transform::from_translation(position)
                    .with_scale(Vec3::splat(rand::random::<f32>() * 0.5 + 0.5)),
                ..default()
            })
            .insert(Organism)
            .insert(Velocity(
                rand_direction * (rand::random::<f32>() * 2.0 - 1.0),
            ));
    });
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        transform.translation += velocity.0 * time.delta_seconds();
    }
}
