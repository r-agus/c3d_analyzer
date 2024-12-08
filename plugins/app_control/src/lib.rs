mod file_drop;
mod mouse_keyboard;

use bevy::{asset::AssetMetaCheck, prelude::*}; 
use bevy_c3d_mod::*;
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_web_file_drop::WebFileDropPlugin;
use config_plugin::{parse_config, C3dConfigPlugin, ConfigC3dAsset, ConfigFile, ConfigState};

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
            .add_plugins(C3dConfigPlugin)
            .add_systems(Startup, setup)
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
            .add_systems(Update, (traces_event_orchestrator, despawn_all_markers_event))
            .add_systems(Update, (change_frame_rate, change_config))
            .add_event::<MarkerEvent>()
            .add_event::<TraceEvent>()
            .add_event::<MilestoneEvent>()
            .add_event::<ReloadRegistryEvent>()
            .init_resource::<AppState>()
            .init_resource::<GuiSidesEnabled>()
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

#[derive(Clone, Default, Debug)]
/// TraceInfo contains the information of the traces to be represented.
/// A trace is the representation of a point along the frames in a given range, with no time information.
/// start_frame: The frame where the trace starts.
/// end_frame: The frame where the trace ends.
/// points: The points that are part of the trace.
pub struct TraceInfo {
    pub start_frame: f32,
    pub end_frame: f32,
    pub points: Vec<String>,
}

#[derive(Event)]
/// MarkerEvent contains the events related to the markers.
pub enum MarkerEvent {
    DespawnAllMarkersEvent,
}

#[derive(Event)]
/// TraceEvent contains the events related to the traces.
pub enum TraceEvent {
    UpdateTraceEvent,
    DespawnTraceEvent(String),
    DespawnAllTracesEvent,
}

#[derive(Event)]
/// MilestoneEvent contains the events related to the milestones.
pub enum MilestoneEvent {
    AddMilestoneFromC3dEvent(usize),
    RemoveMilestoneEvent(usize),
    RemoveAllMilestonesEvent,
}

#[derive(Event)]
pub struct ReloadRegistryEvent;

#[derive(Resource, Default, Debug)]
/// GuiSidesEnabled contains the information of the GUI sides that are enabled.
pub struct GuiSidesEnabled {
    /// The timeline contains the path of the c3d, the frame slider and the play/pause button.
    pub timeline: bool,
    /// The graphs contains the variation of a point among the frames, for example, the position of a marker.
    pub graphs: bool,
}


#[derive(Component)]
/// This is the marker that represents the points in the C3D file, with its label
pub struct Marker(pub String);

#[derive(Component)]
/// This represents the joins between the points in the C3D file. It contains the labels of the points that are joined.
pub struct Join(String, String);

#[derive(Component)]
/// This represents a vector. It contains the labels of the points that are joined. First point is the origin, second point is the vector, third parameter is the scale.
pub struct Vector(Marker, Marker, f64);

#[derive(Component)]
/// This is the trace of a point along the frames
pub struct Trace(pub String);

#[derive(Component)]
/// This is a bunch of markers (parent of Marker)
pub struct C3dMarkers;  


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

impl TraceInfo {
    pub fn default() -> Self {
        TraceInfo {
            start_frame: 0.0,
            end_frame: 0.0,
            points: Vec::new(),
        }
    }

    pub fn is_trace_added (
        &self,
        trace: String,
    ) -> bool {
        self.points.contains(&trace)
    }

    pub fn add_point(&mut self, point: String) {
        self.points.push(point);
    }

    pub fn remove_point(&mut self, point: String) {
        self.points.retain(|x| x != &point);
    }
}

fn setup(
    mut state: ResMut<AppState>,
    mut gui: ResMut<GuiSidesEnabled>,
) {
    state.frame = 0;
    state.c3d_path =  "golpeo3.c3d".to_string();
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

fn spawn_marker(
    label: &str,
    current_config: &str,
    config: &Option<ConfigFile>,
    parent: Entity,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
){
    let matrix = Mat4::from_scale_rotation_translation(
        Vec3::new(1.0, 1.0, 1.0),
        Quat::from_rotation_y(0.0),
        Vec3::new(0.0, 0.0, 0.0),
    );
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(
                // Obtain radius from get_point_size
                Sphere::new(match config.as_ref() {
                    Some(config) => {
                        if let Some(size) = config.get_point_size(label, current_config) {
                            0.014 * size as f32
                        } else {
                            0.014
                        }
                    }
                    None => { 0.014 }
                })
                .mesh(),
            ),
            material: materials.add(StandardMaterial {
                // Obtain color from get_point_color
                base_color: match config.as_ref() {
                    Some(config) => {
                        if let Some(color) = config.get_point_color(label, current_config){
                            if color.len() == 3 {
                                Color::srgb_u8(color[0], color[1], color[2])
                            } else if color.len() == 4 {
                                Color::srgba_u8(color[0], color[1], color[2], color[3])
                            } else {
                                Color::srgb(0.0, 0.0, 1.0)
                            }
                        } else {
                            Color::srgb(0.0, 0.0, 1.0)
                        }
                    }
                    None => { Color::srgb(0.0, 0.0, 1.0) }
                },
                ..default()
            }),
            transform: Transform::from_matrix(matrix),
            visibility: match config.as_ref() {
                Some(config) => {
                    if config.contains_point_regex(current_config, label) {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    }
                }
                None => { Visibility::Visible }
            },
            ..default()
        },
        Marker(label.to_string()),
    )).set_parent(parent);
}

fn spawn_vectors_in_config(
    current_config: &str,
    config_file: &ConfigFile,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
){
    if config_file.get_config(current_config).is_some(){
        if let Some(vectors) = config_file.get_config(current_config).unwrap().get_vectors(){
            for (point, vector) in vectors {
                let mut cone_mesh = Mesh::from(Cone {
                    radius: 0.05,
                    height: 0.2,
                });
                let mut cylinder_mesh = Mesh::from(Cylinder::new(
                    0.01,
                    1.0,    
                ));

                // Extract and modify positions
                if let Some(positions) = cone_mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                    let modified_positions: Vec<[f32; 3]> = positions
                        .as_float3()
                        .unwrap_or(&[[0.0, 0.0, 0.0]])
                        .iter()
                        .map(|&[x, y, z]| [x, y + 0.5, z]) // 0.5 = cylinder height / 2, to place the cone on top of the cylinder (0 is the center of the cylinder)
                        .collect();

                    // Replace the positions attribute
                    cone_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, modified_positions);
                }
                
                cylinder_mesh.merge(&cone_mesh);

                commands.spawn((
                    PbrBundle {
                        mesh: meshes.add(cylinder_mesh),
                        material: materials.add(StandardMaterial {
                        base_color: Color::srgb_u8(255, 220, 0),
                        ..default()
                            }),
                    transform: Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
                    visibility: Visibility::Visible,
                    ..default()
                    }, 
                    Vector(Marker(point.clone()), Marker(vector.0.clone()), vector.1.clone())));
                commands.spawn((
                    PbrBundle {
                        mesh: meshes.add(cone_mesh),
                        material: materials.add(StandardMaterial {
                            base_color: Color::srgb_u8(255, 220, 0),
                            ..default()
                        }),
                        transform: Transform::from_translation(Vec3::new(0.0, vector.1 as f32, 0.0)),
                        visibility: Visibility::Visible,
                        ..default()
                    },
                    Vector(Marker(point.clone()), Marker(vector.0.clone()), vector.1.clone())));
            }
        }
    }
}


fn load_c3d(
    mut c3d_events: EventReader<C3dLoadedEvent>,
    mut milestones_events: EventWriter<MilestoneEvent>,
    c3d_state: ResMut<C3dState>,
    c3d_assets: Res<Assets<C3dAsset>>,
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

        let c3d_asset = c3d_assets.get(&c3d_state.handle);
        let points = 
            commands
                .spawn((
                    PbrBundle {
                        visibility: Visibility::Hidden,
                        ..default()
                    },
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
                for label in &asset.c3d.points.labels {
                    spawn_marker(label, current_config, &config_file, points, &mut commands, &mut meshes, &mut materials); 
                }

                let current_config = app_state.current_config.clone().unwrap_or_default();
                app_state.frame_rate = Some(asset.c3d.points.frame_rate);
                println!("Frame rate: {:?}", asset.c3d.points.frame_rate);
                
                let num_frames = asset.c3d.points.size().0;
                app_state.num_frames = num_frames;
                app_state.traces.end_frame = num_frames as f32 / 20.0; 

                if app_state.fixed_frame_rate.is_none() {
                    app_state.fixed_frame_rate = Some(asset.c3d.points.frame_rate as f64);
                }

                // Spawn joins
                if let Some(config_file) = config_file {
                    let config = config_file.get_config(app_state.current_config.as_deref().unwrap_or("")).unwrap();
                    config.get_joins().into_iter().for_each(|joins| {
                        joins.into_iter().for_each(|join| {
                            for i in 0..join.len() - 1 {
                                let line_thickness = config_file.get_line_thickness(&join[i], &join[i+1], &current_config).unwrap_or(0.01) as f32;
                                let line_color = config_file.get_join_color(&join[i], &join[i+1], &current_config).unwrap_or(vec![0, 255, 0]);
                                commands.spawn((
                                PbrBundle {
                                    mesh: meshes.add(
                                        Cylinder::new(
                                                    if line_thickness > 0.01 { line_thickness * 0.01 } else { 0.01 },
                                            1.0)
                                    ),
                                    material: materials.add(StandardMaterial {
                                        base_color: if line_color.len() == 3 {
                                                        Color::srgb_u8(line_color[0], line_color[1],line_color[2])
                                                    } else if line_color.len() == 4 {
                                                        Color::srgba_u8(line_color[0], line_color[1], line_color[2], line_color[3])
                                                    } else{
                                                        Color::srgb_u8(0, 127, 0)
                                                    },
                                        ..default()
                                    }),
                                    transform: Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
                                    visibility: Visibility::Visible,
                                    ..default()
                                }, Join(join[i].clone(), join[i+1].clone())
                            ));
                        }});
                    });
                }

                // Send milestones to the GUI
                for milestone in asset.c3d.events.iter() {
                    milestones_events.send(MilestoneEvent::AddMilestoneFromC3dEvent((milestone.time * app_state.frame_rate.unwrap_or(1.0)) as usize));
                } 

                println!("C3D loaded");
            }
            None => {
                println!("C3D not loaded");
            }
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
            
            //state.num_frames = num_frames;

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
            }
            state.frame += 1;
            if state.frame >= num_frames {
                state.frame = 0;
            }   
        }
        None => {}
    }
}

pub fn represent_joins(
    markers_query: Query<(&Marker, &Transform)>,
    mut joins_query: Query<(&mut Transform, &Join), Without<Marker>>,
    c3d_state: Res<C3dState>,
    c3d_assets: Res<Assets<C3dAsset>>,
) {
    let asset = c3d_assets.get(&c3d_state.handle);

    match asset {
        Some(_asset) => {
            for (mut transform, join) in joins_query.iter_mut() {
                let marker1 = get_marker_position_on_frame(&join.0, &markers_query);
                let marker2 = get_marker_position_on_frame(&join.1, &markers_query);
                match (marker1, marker2) {
                    (Some(marker1), Some(marker2)) => {
                        let position = (marker1 + marker2) / 2.0;
                        let length = (marker1 - marker2).length();
                        let direction = (marker1 - marker2).normalize();
                        let rotation = Quat::from_rotation_arc(Vec3::Y, direction);
                        let scale = Vec3::new(0.5, length, 0.5);
                        transform.translation = position;
                        transform.rotation = rotation;
                        transform.scale = scale;
                    }
                    _ => {
                        //println!("Error: Marker not found {:?} - {:?}", join.0, join.1); // TODO: Despawn the join
                    }
                }
            }      
        },
        None => {}
    }
}

pub fn represent_vectors(
    markers_query: Query<(&Marker, &Transform)>,
    mut vectors_query: Query<(&Vector, &mut Transform, &mut Visibility), Without<Marker>>,
    c3d_state: Res<C3dState>,
    c3d_assets: Res<Assets<C3dAsset>>,
){
    let asset = c3d_assets.get(&c3d_state.handle);

    match asset {
        Some(_asset) => {
            for (vector, mut transform, mut visibility) in vectors_query.iter_mut() {
                let marker1 = get_marker_position_on_frame(&vector.0.0, &markers_query);
                let marker2 = get_marker_position_on_frame(&vector.1.0, &markers_query);
                match (marker1, marker2) {
                    (Some(marker1), Some(marker2)) => {
                        let length = 50.0 * marker2.length() * vector.2 as f32;
                        let direction = marker2.normalize_or_zero();
                        let position = marker1 + direction * length / 2.0;
                        let rotation = Quat::from_rotation_arc(Vec3::Y, direction);
                        let scale = Vec3::new(1.0, length/vector.2 as f32, 1.0);
                        transform.translation = position;
                        transform.rotation = rotation;
                        transform.scale = scale;
                        if marker2.length() > 0.0005 { // Avoids anoying plate (cone base) when vector is too small
                            *visibility = Visibility::Visible;
                        } else {
                            *visibility = Visibility::Hidden;
                        }
                    }
                    _ => {

                    }
                }
            }      
        },
        None => {}
    }
}

/// Orchestrates the events related to the markers
pub fn traces_event_orchestrator(
    mut events: EventReader<TraceEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: ResMut<AppState>,
    c3d_state: Res<C3dState>,
    c3d_assets: Res<Assets<C3dAsset>>,
    query_positions: Query<(&Marker, &Transform)>,
    query_delete_trace: Query<(Entity, &Trace)>
){
    //for trace_event in events.read() {
    if let Some(trace_event) = events.read().last() {
        match trace_event {
            TraceEvent::UpdateTraceEvent => {
                despawn_all_traces(&mut commands, query_delete_trace);
                represent_traces_event(&mut commands, &mut meshes, &mut materials, &state, &c3d_state, &c3d_assets, &query_positions);
            }
            TraceEvent::DespawnAllTracesEvent => {
                delete_all_traces_event(&mut commands, &mut state, query_delete_trace);
            }
            TraceEvent::DespawnTraceEvent(trace) => {
                delete_trace_event(&mut commands, state, query_delete_trace,trace);
            }
        }
    }
}

fn represent_traces_event(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    state: &ResMut<AppState>,
    c3d_state: &Res<C3dState>,
    c3d_assets: &Res<Assets<C3dAsset>>,
    query_positions: &Query<(&Marker, &Transform)>,
) {
    for point in &state.traces.points {
        let positions = get_marker_position_on_frame_range(point, &c3d_state, &c3d_assets, &query_positions, state.traces.start_frame as usize, state.traces.end_frame as usize);
        match positions {
            Some(positions) => {
                for position in positions {
                    commands.spawn((
                        PbrBundle {
                            mesh: meshes.add(
                                Sphere::new(0.005).mesh()
                            ),
                            material: materials.add(StandardMaterial {
                                base_color: Color::srgb_u8(49, 0, 69),
                                ..default()
                            }),
                            transform: Transform::from_translation(position),
                            visibility: Visibility::Visible,
                            ..default()
                        },    
                        Trace(point.clone()),
                    ));
                }
            }
            None => {
                println!("Error: Trace not found {:?}", point);
            }
        }
    }
    
}

fn delete_trace_event(
    commands: &mut Commands,
    mut state: ResMut<AppState>,
    query_traces: Query<(Entity, &Trace)>,
    delete_trace: &String,
) {
    
    for (entity, trace) in query_traces.iter() {
        let target_trace = delete_trace.clone();
        if trace.0 == target_trace {
            commands.entity(entity).despawn_recursive();
        }
    }
    state.remove_point_from_trace(delete_trace.clone());
    println!("Trace removed: {:?}", delete_trace);
}

fn delete_all_traces_event(
    commands: &mut Commands,
    state: &mut ResMut<AppState>,
    query_traces: Query<(Entity, &Trace)>
) {
    for (entity, _) in query_traces.iter() {
        commands.entity(entity).despawn_recursive();
    }
    state.traces.points.clear();   
}

#[inline]
fn despawn_all_traces(
    commands: &mut Commands,
    query_traces: Query<(Entity, &Trace)>,
) {
    for (entity, _) in query_traces.iter() {
        commands.entity(entity).despawn_recursive();
    }
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

/// Obtain the position of a marker in current frame
pub fn get_marker_position_on_frame(
    label: &str,
    markers_query: &Query<(&Marker, &Transform)>,
) -> Option<Vec3> {
    for (marker, transform) in markers_query.iter() {
        if marker.0 == label {
            return Some(transform.translation);
        } 
    }
    None
}

pub fn get_marker_position_on_all_frames(
    label: &str,
    c3d_state: &Res<C3dState>,
    c3d_assets: &Res<Assets<C3dAsset>>,
    query: &Query<(&Marker, &Transform)>,
) -> Option<Vec<Vec3>> {
    let asset = c3d_assets.get(&c3d_state.handle);
    match asset {
        Some(asset) => {
            let point_data = &asset.c3d.points;
            let num_frames = point_data.size().0;

            return get_marker_position_on_frame_range(label, c3d_state, c3d_assets, query, 0, num_frames);
        }
        None => { return None; }
    }
}

pub fn get_marker_position_on_frame_range(
    label: &str,
    c3d_state: &Res<C3dState>,
    c3d_assets: &Res<Assets<C3dAsset>>,
    query: &Query<(&Marker, &Transform)>,
    start_frame: usize,
    end_frame: usize,
) -> Option<Vec<Vec3>>{
    let asset = c3d_assets.get(&c3d_state.handle);
    match asset {
        Some(asset) => {
            let point_data = &asset.c3d.points;
            let num_frames = point_data.size().0;
            let mut i = 0;
            let mut positions = Vec::new();

            query.iter().for_each(|(marker, _)| {
                if marker.0.split("::").any(|l| {l == label}) {
                    if (start_frame > num_frames) || (end_frame > num_frames) || (start_frame > end_frame) {  // Check if the frames are valid. Start and end are usize, so they can't be negative.
                        println!("Error: Invalid frame range");
                        return;
                    }
                    for frame in start_frame..end_frame {
                        positions.push(Vec3::new(
                            point_data[(frame, i)][0] as f32 / 1000.0, // frame, point_idx, x/y/z
                            point_data[(frame, i)][1] as f32 / 1000.0,
                            point_data[(frame, i)][2] as f32 / 1000.0,
                        ));
                    }
                }
                i += 1;
            });
            return Some(positions);
        }
        None => { return None; }
    }
}

fn despawn_all_markers(
    commands: &mut Commands,
    query_markers: &Query<(Entity, &C3dMarkers)>,
) {
    for (entity, _) in query_markers.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn despawn_all_markers_event(
    mut delete_all_markers_event: EventReader<MarkerEvent>,
    mut commands: Commands,
    query_c3d_markers: Query<(Entity, &C3dMarkers)>,
) {
    if let Some(marker_event) = delete_all_markers_event.read().last() {
        match marker_event {
            MarkerEvent::DespawnAllMarkersEvent => {
                println!("Despawning all markers");
                despawn_all_markers(&mut commands, &query_c3d_markers);
            },
            //_ => {},
        }
    }
}

fn despawn_all_vectors(
    commands: &mut Commands,
    query_vectors: &Query<(Entity, &Vector)>,
) {
    for (entity, _) in query_vectors.iter() {
        commands.entity(entity).despawn_recursive();
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
    
    // TODO: Despawn all joins in separate function
    for (entity, _) in query_joins.iter() {
        commands.entity(entity).despawn_recursive();
    }

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