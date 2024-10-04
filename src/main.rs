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

    let translation = Vec3::new(0., -3.5, 1.0);
        
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                clear_color: Color::srgb(0.8, 0.8, 0.8).into(),
                ..default()
            },
            transform: Transform::from_translation(translation)
                .looking_at(Vec3::new(0., 0., 1.), Vec3::Z),
            ..default()
        },
        PanOrbitCamera{
            ..default()
        }, 
    ));
}