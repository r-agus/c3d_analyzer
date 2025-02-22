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
        .run();
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
