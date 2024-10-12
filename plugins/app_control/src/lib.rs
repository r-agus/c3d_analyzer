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
            .add_systems(Update, (represent_points)
                .run_if(|state: Res<AppState>| -> bool { (state.file_loaded && state.play) || state.render_frame })
                .run_if(|render_at_fixed_frame_rate: Res<AppState>| -> bool { render_at_fixed_frame_rate.render_at_fixed_frame_rate.is_none() }))
            .add_systems(FixedUpdate, (represent_points)
                .run_if(|state: Res<AppState>| -> bool { (state.file_loaded && state.play) || state.render_frame })
                .run_if(|render_at_fixed_frame_rate: Res<AppState>| -> bool { render_at_fixed_frame_rate.render_at_fixed_frame_rate.is_some() }))
            .init_resource::<AppState>()
            .init_resource::<GuiSidesEnabled>()
            .insert_resource(Time::<Fixed>::from_hz(250.));
    }
}

#[derive(Resource, Default, Debug)]
/// AppState contains the relevant information of the c3d file to be loaded and rendered, and the way of representing it.
pub struct AppState {
    /// Current frame
    pub frame: usize,
    /// Number of frames in the c3d file
    pub num_frames: usize,
    /// Path to the c3d file
    pub path: String,
    /// Reload the c3d file. Used to reload the c3d file when the path changes.
    pub reload: bool,
    /// File loaded. Used to know if the c3d file is loaded.
    pub file_loaded: bool,
    /// Play the animation
    pub play: bool,
    /// Send a order to render the frame. Ignores the play state. Must set manually to true every frame, when render is done it is automatically false.
    pub render_frame: bool,
    /// Frame rate of the c3d. You should not modify this value. To adjust the representation speed use render_at_fixed_frame_rate.
    pub frame_rate: Option<f32>,
    /// Frame rate of the animation. None means maximun of your hardware (typically 60Hz of the screen). Fixed is to match the c3d file frame rate, or any other frame rate. May loose information if the frame rate is higher than your hardware maximun.
    pub render_at_fixed_frame_rate: Option<f64>,
}

impl AppState {
    pub fn default() -> Self {
        AppState {
            frame: 0,
            num_frames: 0,
            path: "".to_string(),
            reload: false,
            file_loaded: false,
            play: false,
            render_frame: false,
            frame_rate: None,
            render_at_fixed_frame_rate: None,
        }
    }
}

#[derive(Resource, Default, Debug)]
/// GuiSidesEnabled contains the information of the GUI sides that are enabled.
pub struct GuiSidesEnabled {
    /// The inspector contains the hierarchy of the entities (world) and the properties of the selected entity.
    pub hierarchy_inspector: bool,
    /// The timeline contains the path of the c3d, the frame slider and the play/pause button.
    pub timeline: bool,
}


#[derive(Component)]
/// This is the marker that represents the points in the C3D file
pub struct Marker;      

#[derive(Component)]
/// This is a bunch of markers (parent of Marker)
pub struct C3dMarkers;  
    

fn setup(
    mut state: ResMut<AppState>,
    mut gui: ResMut<GuiSidesEnabled>,
) {
    state.frame = 0;
    state.path =  "walk.c3d".to_string();
    state.reload = true;
    state.file_loaded = true;
    state.play = true;
    state.render_at_fixed_frame_rate = Some(250.);

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
    mut app_state: ResMut<AppState>,
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
                app_state.frame_rate = Some(asset.c3d.points.frame_rate);
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

fn change_frame_rate(
    state: Res<AppState>,
    mut time: ResMut<Time<Fixed>>,
) {
    match state.render_at_fixed_frame_rate {
        Some(frame_rate) => {
            time.set_timestep_hz(frame_rate as f64);
        }
        None => {}
    }
}