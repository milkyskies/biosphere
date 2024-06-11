use bevy::{input::mouse::MouseWheel, prelude::*, window::PrimaryWindow};

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct CameraZoom(f32);

#[derive(Component)]
struct CameraPan {
    is_panning: bool,
    last_position: Vec2,
}

#[derive(Resource, Default)]
struct CursorWorldPosition(Vec2);

#[derive(Resource, Default)]
struct CursorWindowPosition(Vec2);

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CursorWorldPosition::default())
            .insert_resource(CursorWindowPosition::default())
            .add_systems(Startup, setup_camera)
            .add_systems(Update, (handle_zoom, handle_panning, cursor_system));
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle::default(),
        MainCamera,
        CameraZoom(1.0),
        CameraPan {
            is_panning: false,
            last_position: Vec2::new(0.0, 0.0),
        },
    ));
}

fn handle_zoom(
    mut query: Query<(&mut Transform, &mut CameraZoom), With<Camera>>,
    mut scroll_evr: EventReader<MouseWheel>,
) {
    for ev in scroll_evr.read() {
        for (mut transform, mut zoom) in &mut query {
            zoom.0 *= 1.0 - ev.y * 0.1;
            zoom.0 = zoom.0.clamp(0.05, 10.0);
            transform.scale = Vec3::splat(zoom.0);
        }
    }
}

fn handle_panning(
    mut cursor_moved_events: EventReader<CursorMoved>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    cursor_window_position: Res<CursorWindowPosition>,
    mut query: Query<(&mut Transform, &mut CameraPan, &CameraZoom), With<MainCamera>>,
) {
    let (mut transform, mut camera_pan, zoom) = query.single_mut();

    if mouse_button_input.just_pressed(MouseButton::Left) {
        camera_pan.is_panning = true;
        camera_pan.last_position = cursor_window_position.0;
    }

    if mouse_button_input.just_released(MouseButton::Left) {
        camera_pan.is_panning = false;
    }

    if camera_pan.is_panning {
        for event in cursor_moved_events.read() {
            let delta = event.position - camera_pan.last_position;
            transform.translation.x -= delta.x * zoom.0;
            transform.translation.y += delta.y * zoom.0;
            camera_pan.last_position = event.position;
        }
    }
}

fn cursor_system(
    mut cursor_world_position: ResMut<CursorWorldPosition>,
    mut cursor_window_position: ResMut<CursorWindowPosition>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let (camera, camera_transform) = camera_query.single();
    let window = window_query.single();

    if let Some(cursor_position) = window.cursor_position() {
        cursor_window_position.0 = cursor_position;

        if let Some(world_position) = camera
            .viewport_to_world(camera_transform, cursor_position)
            .map(|ray| ray.origin.truncate())
        {
            cursor_world_position.0 = world_position;
        }
    }
}
