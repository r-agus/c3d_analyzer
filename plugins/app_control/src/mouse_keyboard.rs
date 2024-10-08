use crate::*;

pub fn keyboard_controls (
    keyboard: Res<ButtonInput<KeyCode>>,
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
                gui_state.hierarchy_inspector = !gui_state.hierarchy_inspector;
            }
            KeyCode::KeyT => {
                gui_state.timeline = !gui_state.timeline;
            }
            _ => {}
        }
    }    
}