use crate::*;

pub fn keyboard_controls (
    keyboard: Res<ButtonInput<KeyCode>>,
    config_state: Res<ConfigState>,
    config_assets: Res<Assets<ConfigC3dAsset>>,
    mut state: ResMut<AppState>,
    mut gui_state: ResMut<GuiSidesEnabled>,
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
            KeyCode::KeyT => {
                gui_state.timeline = !gui_state.timeline;
            }
            KeyCode::AltLeft => {
                state.render_at_fixed_frame_rate = !state.render_at_fixed_frame_rate;
            }
            KeyCode::Numpad1 | KeyCode::Digit1 => {
                let config_state = config_assets.get(&config_state.handle);
                if let Some(config_state) = config_state {
                    state.current_config = config_state.config.get_all_config_names().first().cloned(); // Reminder: config is a HashMap<String, Config>, so the order is not guaranteed.
                    state.change_config = true;
                    println!("First config: {:?}", state.current_config);
                }                
            }
            KeyCode::Numpad9 | KeyCode::Digit9 => {
                let config_state = config_assets.get(&config_state.handle);
                if let Some(config_state) = config_state {
                    state.current_config = config_state.config.get_all_config_names().last().cloned();
                    state.change_config = true;
                    println!("Last config: {:?}", state.current_config);
                }
            }
            _ => {}
        }
    }    
}