use bevy::input::mouse::{MouseMotion, MouseWheel};

use crate::*;

#[derive(Component)]
pub(crate) struct CustomOrbitCamera {
    pub center: Vec3,
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
}


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
            KeyCode::F6 => {
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

pub(crate) fn update_orbit_camera(
    mut cameras: Query<(&mut Transform, &mut CustomOrbitCamera)>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut motion_evr: EventReader<MouseMotion>,
    mut wheel_evr: EventReader<MouseWheel>,
    mut egui_context: bevy_egui::EguiContexts,
) {
    if egui_context.ctx_mut().wants_pointer_input() {
        return;
    }

    let (mut transform, mut orbit) = cameras.single_mut();

    let mut delta_yaw = 0.0;
    let mut delta_pitch = 0.0;
    let mut delta_pan = Vec2::ZERO;

    for ev in motion_evr.read() {
        if mouse.pressed(MouseButton::Left) {
            delta_yaw -= ev.delta.x * 0.005;
            delta_pitch -= ev.delta.y * 0.005;
        }
        if mouse.pressed(MouseButton::Right) {
            delta_pan += ev.delta;
        }
    }

    for ev in wheel_evr.read(){
        orbit.distance -= ev.y * 0.5;
    }

    orbit.yaw += delta_yaw;
    orbit.pitch = (orbit.pitch + delta_pitch).clamp(-std::f32::consts::FRAC_PI_2, 0.0);

    let base_rotation = Quat::from_xyzw(std::f32::consts::SQRT_2 / 2.0, 0.0, 0.0, std::f32::consts::SQRT_2 / 2.0).normalize();

    let corrected_up = base_rotation * Vec3::Y; 
    let yaw_rotation = Quat::from_axis_angle(corrected_up, orbit.yaw);
    let pitch_rotation = Quat::from_axis_angle(Vec3::X, orbit.pitch);

    let final_rotation = yaw_rotation * base_rotation * pitch_rotation;

    if delta_pan != Vec2::ZERO {
        let pan_speed = 0.0025;
        let right = final_rotation * Vec3::X;
        let up = final_rotation * Vec3::Y;
        orbit.center += right * (-delta_pan.x * pan_speed) + up * (delta_pan.y * pan_speed);
    }

    let initial_offset = final_rotation * Vec3::new(0.0, 0.0, orbit.distance);
    transform.translation = orbit.center + initial_offset;
    transform.rotation = final_rotation;
}

fn get_config_index(c3d_asset: &ConfigC3dAsset, idx: usize) -> Option<String> {
    if c3d_asset.config.get_all_config_names().len() > idx {
        Some(c3d_asset.config.get_all_config_names()[idx].clone())
    } else {
        None
    }
}