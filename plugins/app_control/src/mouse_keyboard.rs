use crate::*;

pub fn keyboard_controls (
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<AppState>,
    query_points: Query<(&C3dMarkers, &Children)>,          // Points and their children (Markers)
    query_markers: Query<(&mut Transform, &Marker)>,
    c3d_state: Res<C3dState>,
    c3d_assets: Res<Assets<C3dAsset>>,
){
    if keyboard.just_pressed(KeyCode::Space) {
        state.play = !state.play;
    }

    if keyboard.just_pressed(KeyCode::ArrowLeft){
        state.frame = state.frame.saturating_sub(2);            // markers() adds 1 to state.frame  
        represent_points(state, query_points, query_markers, c3d_state, c3d_assets);           // render the markers
    } else if keyboard.just_pressed(KeyCode::ArrowRight) {
        state.frame = state.frame.saturating_add(0);
        represent_points(state, query_points, query_markers, c3d_state, c3d_assets);           // render the markers
    }
}