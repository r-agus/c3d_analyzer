use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use gui_plugin::GUIPlugin;
use control_plugin::ControlPlugin;

fn main() {
    App::new()
        .add_plugins(ControlPlugin)
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(GUIPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn a light and the camera
    commands.spawn(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 3.0)),
        ..default()
    });
    
    commands.insert_resource(AmbientLight {
        brightness: 0.3,
        ..default()
    });

    let translation = Vec3::new(0., -5.0, 5.);
        
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                clear_color: Color::srgb(0.8, 0.8, 0.8).into(),  // 0.22, 0.22, 0.22 is cool (but change points to green)
                ..default()
            },
            transform: Transform::from_translation(translation),
                // .looking_at(Vec3::new(0., 0., 1.), Vec3::Z),
            ..default()
        },
        PanOrbitCamera{
            button_orbit: MouseButton::Left,
            button_pan: MouseButton::Right,
            touch_enabled: true,
            reversed_zoom: false,
            ..default()
        }, 
    ));
}