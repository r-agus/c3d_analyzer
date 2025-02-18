use bevy::prelude::*;
use control_plugin::ControlPlugin;
use gui_plugin::GUIPlugin;

use wasm_bindgen::prelude::*;

fn main() {
    #[cfg(target_family = "wasm")]
    start_program();

    App::new()
        .add_plugins(ControlPlugin)
        .add_plugins(GUIPlugin) // TODO: move this to the control_plugin
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn a light and the camera
    // TODO: Spawn floor
    commands.spawn((
        PointLight { ..default() },
        Transform::from_translation(Vec3::new(0.0, 0.0, 3.0)),
    ));

    commands.insert_resource(AmbientLight {
        brightness: 0.3,
        ..default()
    });
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
