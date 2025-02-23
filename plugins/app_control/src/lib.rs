mod file_drop;
mod mouse_keyboard;
pub mod vectors;
pub mod markers;
pub mod joins;
pub mod traces;

use std::{collections::HashMap, vec};

use bevy::{asset::AssetMetaCheck, prelude::*}; 
use bevy_c3d_mod::*;
use bevy_web_file_drop::WebFileDropPlugin;
use config_plugin::{parse_config, C3dConfigPlugin, ConfigC3dAsset, ConfigFile, ConfigState};
use mouse_keyboard::*;
use vectors::*;
use markers::*;
use joins::*;
use traces::*;

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
            .add_plugins(C3dPlugin)
            .add_plugins(C3dConfigPlugin)
            .add_systems(Startup, setup_environment)
            .add_systems(Startup, init_resources)
            .add_systems(First, file_drop::update_c3d_path.run_if(|state: Res<AppState>| -> bool { state.reload_c3d } ))
            .add_systems(First, file_drop::update_configc3d_path.run_if(|state: Res<AppState>| -> bool { state.reload_config } ))
            .add_systems(Update, (file_drop::file_drop, mouse_keyboard::keyboard_controls))
            .add_systems(Update, load_c3d)
            .add_systems(Update, (represent_points)
                .run_if(|state: Res<AppState>| -> bool { (state.c3d_file_loaded && state.play) || state.render_frame })
                .run_if(|state: Res<AppState>| -> bool { state.fixed_frame_rate.is_none() || !state.render_at_fixed_frame_rate }))
            .add_systems(FixedUpdate, (represent_points)
                .run_if(|state: Res<AppState>| -> bool { (state.c3d_file_loaded && state.play) || state.render_frame })
                .run_if(|state: Res<AppState>| -> bool { state.fixed_frame_rate.is_some() && state.render_at_fixed_frame_rate }))
            .add_systems(Update, (represent_joins, represent_vectors))
            .add_systems(Update, (joins_event_orchestrator, traces_event_orchestrator, vector_event_orchestrator, despawn_all_markers_event))
            .add_systems(Update, (change_frame_rate, change_config))
            .add_systems(Update, update_orbit_camera)
            .add_event::<MarkerEvent>()
            .add_event::<JoinEvent>()
            .add_event::<TraceEvent>()
            .add_event::<VectorEvent>()
            .add_event::<MilestoneEvent>()
            .init_resource::<AppState>()
            .init_resource::<GuiSidesEnabled>()
            .init_resource::<VectorsVisibility>()
            .insert_resource(Time::<Fixed>::from_hz(250.));          // default frame rate, can be changed by the user
        println!("Control Plugin loaded");
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
    pub c3d_path: String,
    /// Path to the configuration file.
    pub config_path: String,
    /// Current configuration of the c3d file. Must be defined in the configuration file.
    pub current_config: Option<String>,
    /// Reload the c3d file. Used to reload the c3d file when the path changes.
    pub reload_c3d: bool,
    /// Reload the configuration file. Used to reload the configuration file when the path changes.
    pub reload_config: bool,
    /// Change the configuration. Used to change the configuration of the c3d file.
    pub change_config: bool,
    /// File loaded. Used to know if the c3d file is loaded.
    pub c3d_file_loaded: bool,
    /// Configuration loaded. Used to know if the configuration file is loaded.
    pub config_loaded: bool,
    /// Play the animation
    pub play: bool,
    /// Send a order to render the frame. Ignores the play state. Must set manually to true every frame, when render is done it is automatically false.
    pub render_frame: bool,
    /// Frame rate of the c3d. You should not modify this value. To adjust the representation speed use render_at_fixed_frame_rate.
    pub frame_rate: Option<f32>,
    /// Frame rate of the animation. Fixed is to match the c3d file frame rate, or any other frame rate. May loose information if the frame rate is higher than your hardware maximun.
    pub fixed_frame_rate: Option<f64>,
    /// Render at fixed frame rate. If true, the representation will be at the fixed frame rate. If false, the representation will be at the Update schedule decides (typically 60 Hz).
    pub render_at_fixed_frame_rate: bool,
    /// Trace information. Contains the information of the traces to be represented.
    pub traces: TraceInfo,
}

#[derive(Event)]
/// MilestoneEvent contains the events related to the milestones.
pub enum MilestoneEvent {
    AddMilestoneFromC3dEvent(usize),
    RemoveMilestoneEvent(usize),
    RemoveAllMilestonesEvent,
}

#[derive(Resource, Default, Debug)]
/// GuiSidesEnabled contains the information of the GUI sides that are enabled.
pub struct GuiSidesEnabled {
    /// The timeline contains the path of the c3d, the frame slider and the play/pause button.
    pub timeline: bool,
    /// The graphs contains the variation of a point among the frames, for example, the position of a marker.
    pub graphs: bool,
}

impl AppState {
    pub fn default() -> Self {
        AppState {
            frame: 0,
            num_frames: 0,
            c3d_path: "".to_string(),
            config_path: "".to_string(),
            reload_c3d: false,
            config_loaded: false,
            c3d_file_loaded: false,
            change_config: false,
            reload_config: false,
            play: false,
            render_frame: false,
            frame_rate: None,
            fixed_frame_rate: None,
            render_at_fixed_frame_rate: false,
            // config: None,
            current_config: None,
            traces: TraceInfo {
                ..default()
            },
        }
    }

    pub fn add_point_to_trace(&mut self, point: String) -> &mut Self {
        self.traces.add_point(point);
        self
    }

    pub fn remove_point_from_trace(&mut self, point: String) -> &mut Self {
        self.traces.remove_point(point);
        self
    }
}

fn init_resources(
    mut state: ResMut<AppState>,
    mut gui: ResMut<GuiSidesEnabled>,
) {
    state.frame = 0;
    state.c3d_path =  "".to_string();
    state.current_config = Some("config1".to_string());
    state.config_path = "config_file.toml".to_string();
    state.reload_c3d = true;
    state.c3d_file_loaded = true;
    state.play = true;
    state.fixed_frame_rate = None;
    state.render_at_fixed_frame_rate = false;

    gui.timeline = true;
    gui.graphs = true;

    println!("Control PluginSetup done");
}

fn setup_environment(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Base
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, [5.0, 5.0].into()))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.3, 0.2))),
        Transform::from_rotation(Quat::from_rotation_x(0.0)),
    ));

    commands.spawn((
        PointLight { ..default() },
        Transform::from_translation(Vec3::new(0.0, 0.0, 3.0)),
    ));

    commands.insert_resource(AmbientLight {
        brightness: 0.3,
        ..default()
    });

    // Set camera
    commands.spawn((
        Camera3d { ..default() },
        Camera {
            clear_color: Color::srgb(0.22, 0.22, 0.22).into(),
            ..default()
        },
        CustomOrbitCamera {
            center: Vec3::new(0.0, -0.1, 0.0),
            distance: 7.0,
            yaw: 0.0,
            pitch: -0.5,
        }
    ));

    spawn_reference_vectors(&mut commands, &mut materials, &mut meshes);
}

fn spawn_reference_vectors(
    commands: &mut Commands, 
    materials: &mut ResMut<Assets<StandardMaterial>>, 
    meshes: &mut ResMut<Assets<Mesh>>
) {
    // Vectors for reference
    let default_cylinder_height = 0.25;
    let mut cone_mesh = Mesh::from(Cone {
        radius: 0.025,
        height: 0.1,
    });
    let mut vector_mesh = Mesh::from(Cylinder::new(
        0.01,
        default_cylinder_height,
    ));

    // Extract and modify positions
    if let Some(positions) = cone_mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        let modified_positions: Vec<[f32; 3]> = positions
            .as_float3()
            .unwrap_or(&[[0.0, 0.0, 0.0]])
            .iter()
            .map(|&[x, y, z]| [x, y + default_cylinder_height/2.0, z]) // cylinder height / 2, to place the cone on top of the cylinder (0 is the center of the cylinder)
            .collect();
        // Replace the positions attribute
        cone_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, modified_positions);
    }

    vector_mesh.merge(&cone_mesh);
    
    let colors = vec![
        Color::srgb_u8(255, 0, 0),
        Color::srgb_u8(0, 255, 0),
        Color::srgb_u8(0, 0, 255),
    ];
    let point = Vec3::new(-2.0, -2.0, 0.0);
    let positions = vec![
        point + Vec3::new(default_cylinder_height / 2.0, 0.0, 0.0),
        point + Vec3::new(0.0, default_cylinder_height / 2.0, 0.0),
        point + Vec3::new(0.0, 0.0, default_cylinder_height / 2.0),
    ];
    let rotations = vec![
        Quat::from_axis_angle(Vec3::Z, -std::f32::consts::FRAC_PI_2),
        Quat::IDENTITY,
        Quat::from_axis_angle(Vec3::X, std::f32::consts::FRAC_PI_2),
    ];
    colors.iter().zip(rotations).zip(positions).all(|((color, rotation), position)| {
        commands.spawn((
            Mesh3d(meshes.add(vector_mesh.clone())),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color.clone(),
                ..default()
            })),
            Transform::from_translation(position).with_rotation(rotation),
        ));
        true
    });
}

fn load_c3d(
    mut c3d_events: EventReader<C3dLoadedEvent>,
    mut milestones_events: EventWriter<MilestoneEvent>,
    c3d_state: ResMut<C3dState>,
    mut c3d_assets: ResMut<Assets<C3dAsset>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    mut app_state: ResMut<AppState>,
    config_state: Res<ConfigState>,
    config_assets: Res<Assets<ConfigC3dAsset>>,
    query_markers: Query<(Entity, &C3dMarkers)>,
) {
    if let Some(_) = c3d_events.read().last() {
        
        despawn_all_markers(&mut commands, &query_markers);

        let c3d_asset = c3d_assets.get_mut(&c3d_state.handle);
        let points = 
            commands
                .spawn((
                    Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                    Visibility::from(Visibility::Hidden),
                    C3dMarkers  // This is a bunch of markers
                ))
                .id();
        let config_asset = config_assets.get(&config_state.handle); // This contains the literal text of the configuration file.
        let current_config = app_state.current_config.as_deref().unwrap_or("");
        let config_file = match config_asset {
            Some(asset) => parse_config(&asset.config_str, false).ok(),
            None => {
                println!("Config not loaded");
                None
            }
        };
        
        match c3d_asset {
            Some(asset) => {
                // Spawn markers
                for label in get_all_labels(&asset.c3d) {
                    spawn_marker(&label, current_config, &config_file, points, &mut commands, &mut meshes, &mut materials);
                }

                let current_config = app_state.current_config.clone().unwrap_or_default();
                app_state.frame_rate = Some(asset.c3d.points.frame_rate);
                
                let num_frames = asset.c3d.points.size().0;
                app_state.num_frames = num_frames;
                app_state.traces.end_frame = num_frames as f32 - 20.0; 

                if app_state.fixed_frame_rate.is_none() {
                    app_state.fixed_frame_rate = Some(asset.c3d.points.frame_rate as f64);
                }

                // Spawn joins
                if let Some(config_file) = config_file {
                    spawn_joins_in_config(&current_config, &config_file, &mut commands, &mut meshes, &mut materials);
                }

                // Send milestones to the GUI
                let start_frame = asset.c3d.points.first_frame as f32;
                for milestone in asset.c3d.events.iter() {
                    let milestone_frame = milestone.time * asset.c3d.points.frame_rate - start_frame;
                    milestones_events.send(MilestoneEvent::AddMilestoneFromC3dEvent(milestone_frame as usize + 1));
                } 

                println!("C3D loaded");
            }
            None => {
                println!("C3D not loaded");
            }
        }
    }
}

pub(crate) fn get_all_labels(
    c3d: &C3d,
) -> Vec<String> {
    let mut labels = c3d.points.labels.clone();
    if let Some(group) = c3d.parameters.get_group("POINT"){
        let other_labels_params = group
            .iter()
            .filter(|(k, _)| k.starts_with("LABEL"))
            .collect::<HashMap<_, _>>();

        let mut other_labels_params = other_labels_params.into_iter().collect::<Vec<_>>();
        other_labels_params.sort_by_key(|(k, _)| *k);

        for (_, param) in other_labels_params {
            match param.data.clone() {
                ParameterData::Char(vec) => {
                    let dim = param.dimensions.clone();
                    let label = vec
                        .chunks(dim[0] as usize)
                        .map(|chunk| chunk.iter().collect::<String>())
                        .collect::<Vec<String>>();
                    let label = label
                        .iter()
                        .map(|s| s.trim().to_string())
                        .collect::<Vec<String>>();
                    labels.extend(label);
                },
                _ => {}
            }
        }
    }
    labels
}

fn change_frame_rate(
    state: Res<AppState>,
    mut time: ResMut<Time<Fixed>>,
) {
    match state.fixed_frame_rate {
        Some(frame_rate) => {
            time.set_timestep_hz(frame_rate as f64);
        }
        None => {}
    }
}

/// Change the configuration of the c3d file. This can be used to change the representation of the c3d file.
fn change_config(
    mut state: ResMut<AppState>,
    mut commands: Commands,
    query_joins: Query<(Entity, &Join)>,
    query_vectors: Query<(Entity, &Vector)>,
    mut ev_loaded: EventWriter<C3dLoadedEvent>,
    config_state: Res<ConfigState>,
    config_assets: Res<Assets<ConfigC3dAsset>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !state.change_config{
        return;
    }
    state.change_config = false;
    
    despawn_all_joins(&mut commands, &query_joins);

    // Load the new configuration. Just need to call load_c3d again. Will be taken into account in the next frame.
    ev_loaded.send(C3dLoadedEvent);

    // Despawn old vectors and spawn new vectors
    despawn_all_vectors(&mut commands, &query_vectors);
    if let Some(config) = config_assets.get(&config_state.handle){
        if let Some(current_config_name) = state.current_config.as_deref() {
            spawn_vectors_in_config(current_config_name, &config.config, &mut commands, &mut meshes, &mut materials);   
        }
    }
}