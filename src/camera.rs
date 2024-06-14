use bevy::{
    input::{gestures::*, keyboard::KeyboardInput, mouse::MouseWheel},
    prelude::*,
    window::PrimaryWindow,
};

pub struct CameraPlugin;

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

#[derive(Resource, Debug, PartialEq)]
enum InputDevice {
    Mouse,
    Touchpad,
}

impl Default for InputDevice {
    fn default() -> Self {
        InputDevice::Mouse
    }
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CursorWorldPosition::default())
            .insert_resource(CursorWindowPosition::default())
            .insert_resource(InputDevice::default())
            .add_systems(Startup, setup_camera)
            .add_systems(
                Update,
                (
                    set_input_device,
                    handle_scroll_zoom,
                    handle_scroll_pan,
                    handle_pinch_zoom,
                    handle_click_pan,
                    cursor_system,
                ),
            );
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

fn handle_scroll_zoom(
    mut query: Query<(&mut Transform, &mut CameraZoom), With<MainCamera>>,
    mut scroll_evr: EventReader<MouseWheel>,
    input_device: Res<InputDevice>,
) {
    if *input_device != InputDevice::Mouse {
        return;
    }

    for event in scroll_evr.read() {
        for (mut transform, mut zoom) in query.iter_mut() {
            zoom.0 *= 1.0 - event.y * 0.1;
            zoom.0 = zoom.0.clamp(0.05, 10.0);
            transform.scale = Vec3::splat(zoom.0);
        }
    }
}

fn handle_pinch_zoom(
    mut query: Query<(&mut Transform, &mut CameraZoom), With<MainCamera>>,
    mut pinch_gesture_evr: EventReader<PinchGesture>,
) {
    for event in pinch_gesture_evr.read() {
        let pinch_scale = event.0;

        for (mut transform, mut zoom) in query.iter_mut() {
            zoom.0 -= pinch_scale;
            transform.scale = Vec3::splat(zoom.0);
        }
    }
}

fn handle_scroll_pan(
    mut query: Query<&mut Transform, With<MainCamera>>,
    mut scroll_evr: EventReader<MouseWheel>,
    input_device: Res<InputDevice>,
) {
    if *input_device != InputDevice::Touchpad {
        return;
    }

    for event in scroll_evr.read() {
        for mut transform in query.iter_mut() {
            transform.translation.x -= event.x;
            transform.translation.y += event.y;
        }
    }
}

fn handle_click_pan(
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

fn set_input_device(mut commands: Commands, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::KeyM) {
        commands.insert_resource(InputDevice::Mouse);
    } else if keyboard_input.just_pressed(KeyCode::KeyT) {
        commands.insert_resource(InputDevice::Touchpad);
    }
}
