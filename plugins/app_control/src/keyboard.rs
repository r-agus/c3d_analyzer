use crate::*;

pub fn keyboard_controls (
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<AppState>,
    query: Query<(&mut Transform, &Marker)>, // to call markers
    c3d_state: ResMut<C3dState>,
    c3d_assets: Res<Assets<C3dAsset>>,
){
    if keyboard.just_pressed(KeyCode::Space) {
        state.play = !state.play;
    }

    if keyboard.just_pressed(KeyCode::ArrowLeft){
        state.frame = state.frame.saturating_sub(2);            // markers() adds 1 to state.frame  
        markers(state, query, c3d_state, c3d_assets);           // render the markers
    } else if keyboard.just_pressed(KeyCode::ArrowRight) {
        state.frame = state.frame.saturating_add(0);
        markers(state, query, c3d_state, c3d_assets);           // render the markers
    }
}