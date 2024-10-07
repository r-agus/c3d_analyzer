use crate::*;

pub fn file_drop(
    mut evr_dnd: EventReader<FileDragAndDrop>,
    mut state: ResMut<AppState>,
) {
    for ev in evr_dnd.read() {
        if let FileDragAndDrop::DroppedFile { window, path_buf } = ev {
            println!("Dropped file with path: {:?}, in window id: {:?}", path_buf, window);
            state.path = path_buf.to_str().unwrap().to_string();
            state.reload = true;
            state.file_loaded = true;
            state.frame = 0;
        }
    }
}

pub fn update_c3d_path(
    mut state: ResMut<AppState>,
    asset_server: Res<AssetServer>,
    mut c3d_state: ResMut<C3dState>,
) {
    if state.reload {
        c3d_state.handle = asset_server.load(state.path.clone());
        state.reload = false;
    }
}