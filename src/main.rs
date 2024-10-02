use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_c3d_mod::*;
use bevy_web_file_drop::WebFileDropPlugin;

fn main() {
    App::new()
        .add_plugins((
            WebFileDropPlugin,                          // Has to load before AssetPlugin (that is loaded by DefaultPlugins)
            DefaultPlugins.set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
        ))
        .add_plugins(C3dPlugin)
        .add_systems(Startup, setup)
        .add_systems(First, update_c3d_path.run_if(|state: Res<State>| -> bool { state.reload } ))
        .add_systems(Update, (file_drop, load_c3d, keyboard_controls))
        .add_systems(Update, (markers).run_if(|state: Res<State>| -> bool { state.file_loaded && state.play }))
        .init_resource::<State>()
        .run();
}

#[derive(Resource, Default, Debug)]
struct State {
    pub frame: usize,
    pub path: String,
    pub reload: bool,
    pub file_loaded: bool,
    pub play: bool,
}

#[derive(Component)]
struct Marker;

fn keyboard_controls (
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<State>,
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

fn file_drop(
    mut evr_dnd: EventReader<FileDragAndDrop>,
    mut state: ResMut<State>,
) {
    for ev in evr_dnd.read() {
        if let FileDragAndDrop::DroppedFile { window, path_buf } = ev {
            println!("Dropped file with path: {:?}, in window id: {:?}", path_buf, window);
            state.path = path_buf.to_str().unwrap().to_string();
            state.reload = true;
            state.file_loaded = true;
            state.frame = 0;
        }
    }
}

// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut state: ResMut<State>,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Default state
    state.frame = 0;
    state.path =  "".to_string();
    state.reload = false;
    state.file_loaded = false;
    state.play = true;

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
        
    commands.spawn((Camera3dBundle {
        camera: Camera {
            clear_color: Color::srgb(0.8, 0.8, 0.8).into(),
            ..Default::default()
        },
        transform: Transform::from_translation(translation)
            .looking_at(Vec3::new(0., 0., 1.), Vec3::Z),
        ..Default::default()
    },));
}

fn update_c3d_path(
    mut state: ResMut<State>,
    mut c3d_state: ResMut<C3dState>,
    asset_server: Res<AssetServer>,
) {
    if state.reload {
        c3d_state.handle = asset_server.load(state.path.clone());
        state.reload = false;
    }
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
                    ));
                }
            }
            None => {}
        }
    }
}

fn markers(
    mut state: ResMut<State>,
    mut query: Query<(&mut Transform, &Marker)>,
    c3d_state: ResMut<C3dState>,
    c3d_assets: Res<Assets<C3dAsset>>,
) {
    let asset = c3d_assets.get(&c3d_state.handle);
    match asset {
        Some(asset) => {
            let point_data = &asset.c3d.points;
            let num_frames = point_data.size().0;
            let mut i = 0;
            for (mut transform, _) in query.iter_mut() {
                transform.translation = Vec3::new(
                    point_data[(state.frame, i)][0] as f32 / 1000.0,
                    point_data[(state.frame, i)][1] as f32 / 1000.0,
                    point_data[(state.frame, i)][2] as f32 / 1000.0,
                );
                i += 1;
            }
            state.frame += 1;
            if state.frame >= num_frames {
                state.frame = 0;
            }
        }
        None => {}
    }
}
