mod file_drop;
mod mouse_keyboard;

use bevy::{asset::AssetMetaCheck, prelude::*}; 
use bevy_c3d_mod::*;
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_web_file_drop::WebFileDropPlugin;
use config_plugin::{parse_config, C3dConfigPlugin, ConfigC3dAsset, ConfigState};

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
            .add_systems(Update, represent_joins)
            .add_systems(Update, (change_frame_rate, change_config))
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
        }
    }
}

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
pub struct Marker(String);      

#[derive(Component)]
/// This represents the joins between the points in the C3D file. It contains the labels of the points that are joined.
pub struct Join(String, String);

#[derive(Component)]
/// This is a bunch of markers (parent of Marker)
pub struct C3dMarkers;  

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
    
    println!("Control PluginSetup done");
}

fn load_c3d(
    mut events: EventReader<C3dLoadedEvent>,
    c3d_state: ResMut<C3dState>,
    c3d_assets: Res<Assets<C3dAsset>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    mut app_state: ResMut<AppState>,
    config_state: Res<ConfigState>,
    config_assets: Res<Assets<ConfigC3dAsset>>,
) {
    if let Some(_) = events.read().last() {
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
        let config = match config_asset {
            Some(asset) => parse_config(&asset.config_str, false).ok(),
            None => {
                println!("Config not loaded");
                None
            }
        };
        
        match c3d_asset {
            Some(asset) => {
                for label in &asset.c3d.points.labels {
                    let matrix = Mat4::from_scale_rotation_translation(
                        Vec3::new(1.0, 1.0, 1.0),
                        Quat::from_rotation_y(0.0),
                        Vec3::new(0.0, 0.0, 0.0),
                    );
                    commands.spawn((
                        PbrBundle {
                            mesh: meshes.add(
                                // Obtain radius from get_point_size
                                Sphere::new(match &config {
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
                                base_color: match &config {
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
                            visibility: match &config {
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
                        Marker(label.clone()),
                    )).set_parent(points);
                }
                let current_config = app_state.current_config.clone().unwrap_or_default();
                app_state.frame_rate = Some(asset.c3d.points.frame_rate);
                println!("Frame rate: {:?}", asset.c3d.points.frame_rate);
                
                if app_state.fixed_frame_rate.is_none() {
                    app_state.fixed_frame_rate = Some(asset.c3d.points.frame_rate as f64);
                }

                if let Some(config_file) = config {
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
                        println!("Error: Marker not found {:?} - {:?}", join.0, join.1);
                    }
                }
            }      
        },
        None => {}
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
    c3d_state: Res<C3dState>,
    c3d_assets: Res<Assets<C3dAsset>>,
    query: Query<(&Transform, &Marker)>,
) -> Option<Vec<Vec3>> {
    let asset = c3d_assets.get(&c3d_state.handle);
    match asset {
        Some(asset) => {
            let point_data = &asset.c3d.points;
            let num_frames = point_data.size().0;
            let mut frame = 0;
            let mut i = 0;
            let mut positions = Vec::new();

            query.iter().for_each(|(_, marker)| {
                if marker.0 == label {
                    for _ in 0..num_frames {
                        positions.push(Vec3::new(
                            point_data[(frame, i)][0] as f32 / 1000.0,
                            point_data[(frame, i)][1] as f32 / 1000.0,
                            point_data[(frame, i)][2] as f32 / 1000.0,
                        ));
                        frame += 1;
                    }
                }
                i += 1;
            });
            return Some(positions);
        }
        None => { return None; }
    }
}

/// Change the configuration of the c3d file. This can be used to change the representation of the c3d file.
fn change_config(
    mut state: ResMut<AppState>,
    mut commands: Commands,
    query_c3d_markers: Query<(Entity, &C3dMarkers)>,
    query_joins: Query<(Entity, &Join)>,
    mut ev_loaded: EventWriter<C3dLoadedEvent>,
) {
    if !state.change_config{
        return;
    }
    state.change_config = false;
    
    // First we need to despawn all the markers (and its parent, C3dMarker), joins
    for (entity, _) in query_c3d_markers.iter() {
        commands.entity(entity).despawn_recursive(); // Also despawns the children (markers)
    }
    for (entity, _) in query_joins.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Then we load the new configuration. Just need to call load_c3d again.
    ev_loaded.send(C3dLoadedEvent);
}