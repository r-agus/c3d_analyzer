mod file_drop;
mod keyboard;

use bevy::{asset::AssetMetaCheck, prelude::*}; 
use bevy_c3d_mod::*;
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
            .add_plugins(C3dPlugin)
            .add_systems(Startup, setup)
            .add_systems(First, file_drop::update_c3d_path.run_if(|state: Res<AppState>| -> bool { state.reload } ))
            .add_systems(Update, (file_drop::file_drop, keyboard::keyboard_controls))
            .add_systems(Update, load_c3d)
            .add_systems(Update, (markers).run_if(|state: Res<AppState>| -> bool { state.file_loaded && state.play }))
            .init_resource::<AppState>();
    }
}

#[derive(Resource, Default, Debug)]
pub struct AppState {
    pub frame: usize,
    pub path: String,
    pub reload: bool,
    pub file_loaded: bool,
    pub play: bool,
}

#[derive(Component)]
struct Marker;

#[derive(Component)]
struct _Points;

fn setup(
    mut state: ResMut<AppState>,
) {
    state.frame = 0;
    state.path =  "walk.c3d".to_string();
    state.reload = true;
    state.file_loaded = true;
    state.play = true;
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
        // let points = commands.spawn(Points).id();
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
    mut state: ResMut<AppState>,
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
            // for(_, mut marker) in query.iter_mut() 
            // let markers = query.get_single_mut();
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
