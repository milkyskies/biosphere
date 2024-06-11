use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    pbr::ClusterConfig,
    prelude::*,
    window::PrimaryWindow,
};

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct CameraZoom {
    distance: f32,
    min_distance: f32,
    max_distance: f32,
}

#[derive(Component)]
struct CameraPan {
    last_position: Vec2,
    is_panning: bool,
}

#[derive(Resource, Default)]
struct CursorWindowPosition(Vec2);

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CursorWindowPosition::default())
            .add_systems(Startup, setup_camera)
            // .add_systems(Update, (cursor_system, handle_zoom))
            .add_systems(Update, (handle_zoom, handle_panning, cursor_system));
    }
}

fn setup_camera(mut commands: Commands) {
    commands
        .spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(-3.0, 5.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            },
            ClusterConfig::Single,
        ))
        .insert(MainCamera)
        .insert(CameraZoom {
            distance: 10.0,
            min_distance: 5.0,
            max_distance: 5000.0,
        })
        .insert(CameraPan {
            last_position: Vec2::ZERO,
            is_panning: false,
        });
}

fn handle_zoom(
    mut query: Query<(&mut Transform, &mut CameraZoom), With<MainCamera>>,
    mut scroll_evr: EventReader<MouseWheel>,
) {
    for ev in scroll_evr.read() {
        for (mut transform, mut zoom) in query.iter_mut() {
            let new_distance =
                (zoom.distance - ev.y * 12.0).clamp(zoom.min_distance, zoom.max_distance);
            transform.translation = transform.translation.normalize() * new_distance;
            zoom.distance = new_distance;
        }
    }
}

fn handle_panning(
    mut query: Query<(&mut Transform, &mut CameraPan), With<MainCamera>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    cursor_window_position: Res<CursorWindowPosition>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
) {
    for (mut transform, mut pan) in query.iter_mut() {
        if mouse_button_input.pressed(MouseButton::Left) {
            if !pan.is_panning {
                pan.is_panning = true;
                pan.last_position = cursor_window_position.0;
            } else {
                let cursor_position = cursor_window_position.0;
                let delta = cursor_position - pan.last_position;
                let rotation_speed = 0.003;

                // Calculate spherical rotation
                let yaw = Quat::from_rotation_y(-delta.x * rotation_speed);
                let pitch = Quat::from_rotation_x(-delta.y * rotation_speed);

                // Apply rotation to the camera's transform
                transform.rotation = yaw * transform.rotation; // Yaw around global Y axis
                transform.rotation = transform.rotation * pitch; // Pitch around local X axis

                // Update the last position for the next frame
                pan.last_position = cursor_position;
            }
        } else {
            pan.is_panning = false;
        }
    }
}

fn cursor_system(
    mut cursor_window_position: ResMut<CursorWindowPosition>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.single();

    if let Some(cursor_position) = window.cursor_position() {
        cursor_window_position.0 = cursor_position;
    }
}
