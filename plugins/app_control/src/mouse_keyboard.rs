use crate::*;

pub fn keyboard_controls (
    keyboard: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
    config_state: Res<ConfigState>,
    config_assets: Res<Assets<ConfigC3dAsset>>,
    mut state: ResMut<AppState>,
    mut gui_state: ResMut<GuiSidesEnabled>,
    
    
    mut despawn_all_markers_event: EventWriter<MarkerEvent>,
){
    if let Some(key) = keyboard.get_just_pressed().next() {
        match key {
            KeyCode::Space => {
                state.play = !state.play;
            }
            KeyCode::ArrowLeft => {
                state.frame = state.frame.saturating_sub(2);  // represent_points increments frame by 1
                state.render_frame = true;
            }
            KeyCode::ArrowRight => {
                state.frame = state.frame.saturating_add(0);
                state.render_frame = true;
            }
            KeyCode::Escape => {
                // TODO: Implement a way to stop the program    
            }
            KeyCode::F5 => {
                println!("Reloading assets");
                asset_server.reload(state.c3d_path.clone());
                asset_server.reload(state.config_path.clone());
            }
            KeyCode::F12 => {
                despawn_all_markers_event.send(MarkerEvent::DespawnAllMarkersEvent);
            }
            KeyCode::KeyG => {
                gui_state.graphs = !gui_state.graphs;
            }
            KeyCode::KeyT => {
                gui_state.timeline = !gui_state.timeline;
            }
            KeyCode::AltLeft => {
                state.render_at_fixed_frame_rate = !state.render_at_fixed_frame_rate;
            }
            KeyCode::Numpad1 | KeyCode::Digit1 => {
                let config_state = config_assets.get(&config_state.handle);
                if let Some(config_state) = config_state {
                    state.current_config = config_state
                        .config
                        .get_all_config_names()
                        .first()
                        .cloned(); 
                    state.change_config = true;
                    println!("First config: {:?}", state.current_config);
                }                
            }
            
            KeyCode::Numpad2 | KeyCode::Digit2 |
            KeyCode::Numpad3 | KeyCode::Digit3 |
            KeyCode::Numpad4 | KeyCode::Digit4 |
            KeyCode::Numpad5 | KeyCode::Digit5 |
            KeyCode::Numpad6 | KeyCode::Digit6 |
            KeyCode::Numpad7 | KeyCode::Digit7 |
            KeyCode::Numpad8 | KeyCode::Digit8 => {
                let config_state = config_assets.get(&config_state.handle);
                if let Some(config_state) = config_state {
                    let idx = match key {
                        KeyCode::Numpad2 | KeyCode::Digit2 => 1,
                        KeyCode::Numpad3 | KeyCode::Digit3 => 2,
                        KeyCode::Numpad4 | KeyCode::Digit4 => 3,
                        KeyCode::Numpad5 | KeyCode::Digit5 => 4,
                        KeyCode::Numpad6 | KeyCode::Digit6 => 5,
                        KeyCode::Numpad7 | KeyCode::Digit7 => 6,
                        KeyCode::Numpad8 | KeyCode::Digit8 => 7,
                        _ => unreachable!(),
                    };
                    let get_config = get_config_index(config_state, idx);
                    if get_config.is_none() {
                        return;
                    }
                    if get_config != state.current_config {
                        state.current_config = get_config;
                        state.change_config = true;
                    }
                }
            }

            KeyCode::Numpad9 | KeyCode::Digit9 => {
                let config_state = config_assets.get(&config_state.handle);
                if let Some(config_state) = config_state {
                    state.current_config = config_state
                        .config
                        .get_all_config_names()
                        .last()
                        .cloned();
                    state.change_config = true;
                    println!("Last config: {:?}", state.current_config);
                }
            }
            _ => {}
        }
    }    
}

fn get_config_index(c3d_asset: &ConfigC3dAsset, idx: usize) -> Option<String> {
    if c3d_asset.config.get_all_config_names().len() > idx {
        Some(c3d_asset.config.get_all_config_names()[idx].clone())
    } else {
        None
    }
}