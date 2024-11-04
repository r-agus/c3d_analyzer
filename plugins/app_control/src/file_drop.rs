use std::ffi::OsStr;

use config_plugin::ConfigState;

use crate::*;

pub fn file_drop(
    mut ev_loaded: EventWriter<C3dLoadedEvent>,
    mut evr_dnd: EventReader<FileDragAndDrop>,
    mut state: ResMut<AppState>,
    query: Query<(Entity, &C3dMarkers)>,
    mut commands: Commands,
) {
    for ev in evr_dnd.read() {
        if let FileDragAndDrop::DroppedFile { window, path_buf } = ev {
            println!("Dropped file with path: {:?}, in window id: {:?}", path_buf, window);
            let extension = path_buf.extension();
            match extension {
                Some(extension) => {
                    if extension == OsStr::new("c3d") {
                        for (entity, _) in query.iter() {
                            commands.entity(entity).despawn_recursive();
                        }
                        state.c3d_path = path_buf.to_str().unwrap().to_string();
                        state.reload_c3d = true;
                        state.c3d_file_loaded = true;
                        state.play = false;
                        state.frame = 0;
                        ev_loaded.send(C3dLoadedEvent);
                    }
                    if extension == OsStr::new("toml") {
                        state.config_path = path_buf.to_str().unwrap().to_string();
                        state.reload_config = true;
                    }
                },
                None => {},
            }
        }
    }
}

pub fn update_c3d_path(
    mut state: ResMut<AppState>,
    asset_server: Res<AssetServer>,
    mut c3d_state: ResMut<C3dState>,
) {
    if state.reload_c3d {
        c3d_state.handle = asset_server.load(state.c3d_path.clone());
        state.reload_c3d = false;
    }
}

pub fn update_configc3d_path(
    mut state: ResMut<AppState>,
    asset_server: Res<AssetServer>,
    mut conf_state: ResMut<ConfigState>,
) {
    if state.reload_config {
        conf_state.handle = asset_server.load(state.config_path.clone());
        state.reload_config = false;
        state.reload_c3d = true;
        println!("Config file reloaded");
    }
}