use bevy::prelude::*;
use control_plugin::ControlPlugin;
use gui_plugin::GUIPlugin;
use bevy_blendy_cameras::{
    BlendyCamerasPlugin, FlyCameraController, OrbitCameraController,
};

use wasm_bindgen::prelude::*;

fn main() {
    #[cfg(target_family = "wasm")]
    start_program();

    App::new()
        .add_plugins(ControlPlugin)
        .add_plugins(BlendyCamerasPlugin)
        .add_plugins(GUIPlugin) // TODO: move this to the control_plugin
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
) {
    // Spawn a light and the camera
    commands.spawn((
        PointLight { ..default() },
        Transform::from_translation(Vec3::new(0.0, 0.0, 3.0)),
    ));

    commands.insert_resource(AmbientLight {
        brightness: 0.3,
        ..default()
    });

    let translation = Vec3::new(0., -5.0, 5.);

    commands.spawn((
        Camera3d { ..default() },
        Camera {
            clear_color: Color::srgb(0.8, 0.8, 0.8).into(), // 0.22, 0.22, 0.22 is cool (but change points to green)
            ..default()
        },
        OrbitCameraController{
            orbit_sensitivity: 0.5,
            pan_sensitivity: 1.0,
            button_orbit: MouseButton::Left,
            button_pan: MouseButton::Right,
            is_enabled: true,
            modifier_pan: None,
            // zoom_to_mouse_position: todo!(),
            auto_depth: false,
            ..default()
        },
        FlyCameraController {
            is_enabled: false,
            ..default()
        },
        Transform::from_translation(translation).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

#[wasm_bindgen]
extern "C" {
    fn notify_start();
}
#[cfg(target_family = "wasm")]
pub fn start_program() {
    // Llamar a la función de JavaScript para indicar el inicio (eliminar pantalla de carga)
    notify_start();
}
