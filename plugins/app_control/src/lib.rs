mod file_drop;
mod mouse_keyboard;

use bevy::{asset::AssetMetaCheck, prelude::*}; 
use bevy_c3d_mod::*;
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_web_file_drop::WebFileDropPlugin;

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((WebFileDropPlugin, DefaultPlugins.set(
                AssetPlugin {
                            meta_check: AssetMetaCheck::Never,
                            ..default()
                        }
                )))
            .add_plugins((C3dPlugin, DefaultPickingPlugins))
            .add_systems(Startup, setup)
            .add_systems(First, file_drop::update_c3d_path.run_if(|state: Res<AppState>| -> bool { state.reload } ))
            .add_systems(Update, (file_drop::file_drop, mouse_keyboard::keyboard_controls))
            .add_systems(Update, load_c3d)
            .add_systems(Update, (represent_points).run_if(|state: Res<AppState>| -> bool { (state.file_loaded && state.play) || state.render_frame }))
            .init_resource::<AppState>()
            .init_resource::<GuiSidesEnabled>();
    }
}

#[derive(Resource, Default, Debug)]
pub struct AppState {
    pub frame: usize,       // Current frame
    pub num_frames: usize,  // Number of frames in the c3d file
    pub path: String,
    pub reload: bool,
    pub file_loaded: bool,
    pub play: bool,         // Play the animation
    pub render_frame: bool, // Send a order to render the frame. Ignores the play state. Must set manually to true every frame.
}

#[derive(Resource, Default, Debug)]
pub struct GuiSidesEnabled {
    pub hierarchy_inspector: bool,
    pub timeline: bool,
}


#[derive(Component)]
pub struct Marker;      // This is the marker that represents the points in the C3D file

#[derive(Component)]
pub struct C3dMarkers;  // This is a bunch of markers (parent of Marker)
    

fn setup(
    mut state: ResMut<AppState>,
    mut gui: ResMut<GuiSidesEnabled>,
) {
    state.frame = 0;
    state.path =  "walk.c3d".to_string();
    state.reload = true;
    state.file_loaded = true;
    state.play = true;

    gui.hierarchy_inspector = false;
    gui.timeline = true;
}

fn load_c3d(
    mut events: EventReader<C3dLoadedEvent>,
    c3d_state: ResMut<C3dState>,
    c3d_assets: Res<Assets<C3dAsset>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    if let Some(_) = events.read().last() {
        let asset = c3d_assets.get(&c3d_state.handle);
        let points = 
            commands
                .spawn((
                    PbrBundle {
                        ..default()
                    },
                    C3dMarkers  // We need C3dMarkers to have certain properties, so use PbrBundle as a base.
                ))
                .id();
        
        match asset {
            Some(asset) => {
                for _ in 0..asset.c3d.points.labels.len() {
                    let matrix = Mat4::from_scale_rotation_translation(
                        Vec3::new(1.0, 1.0, 1.0),
                        Quat::from_rotation_y(0.0),
                        Vec3::new(0.0, 0.0, 0.0),
                    );
                    commands.spawn((
                        PbrBundle {
                            mesh: meshes.add(
                                Sphere::new(0.014).mesh(),
                            ),
                            material: materials.add(StandardMaterial {
                                base_color: Color::srgb_u8(0, 0, 127),
                                ..default()
                            }),
                            transform: Transform::from_matrix(matrix),
                            ..default()
                        },
                        Marker,
                    )).set_parent(points);
                }
            }
            None => {}
        }
    }
}

pub fn represent_points(
    mut state: ResMut<AppState>,
    query_points: Query<(&C3dMarkers, &Children)>,          // C3dMarkers and their children (Markers)
    mut query_markers: Query<(&mut Transform, &Marker)>,
    c3d_state: Res<C3dState>,
    c3d_assets: Res<Assets<C3dAsset>>,
) {
    if state.render_frame {
        state.render_frame = false;
    }

    let asset = c3d_assets.get(&c3d_state.handle);

    match asset {
        Some(asset) => {
            let point_data = &asset.c3d.points;
            let num_frames = point_data.size().0;
            let mut i = 0;
            
            state.num_frames = num_frames;

            for (_points, children) in query_points.iter() {
                for &child in children.iter() {
                    let pos = query_markers.get_mut(child);
                    match pos {
                        Ok((mut transform, _)) => {
                            transform.translation = Vec3::new(
                                point_data[(state.frame, i)][0] as f32 / 1000.0,
                                point_data[(state.frame, i)][1] as f32 / 1000.0,
                                point_data[(state.frame, i)][2] as f32 / 1000.0,
                            );
                            i += 1;
                        }
                        Err(_) => {}
                    }
                }    
                state.frame += 1;
                if state.frame >= num_frames {
                    state.frame = 0;
                }        
            }
        }
        None => {}
    }
}
