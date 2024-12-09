use crate::*;

#[derive(Component)]
/// This is a bunch of markers (parent of Marker)
pub struct C3dMarkers;  

#[derive(Component, Clone, PartialEq)]
/// This is the marker that represents the points in the C3D file, with its label
pub struct Marker(pub String);


#[derive(Event)]
/// MarkerEvent contains the events related to the markers.
pub enum MarkerEvent {
    DespawnAllMarkersEvent,
}

pub(crate) fn spawn_marker(
    label: &str,
    current_config: &str,
    config: &Option<ConfigFile>,
    parent: Entity,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
){
    let marker_mesh = meshes.add(
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
        }).mesh());
    let marker_material = materials.add(StandardMaterial {
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
    });

    let marker_visibility = match config.as_ref() {
        Some(config) => {
            if config.contains_point_regex(current_config, label) {
                Visibility::Visible
            } else {
                Visibility::Hidden
            }
        }
        None => { Visibility::Visible }
    };    
    
    commands.spawn((
        Mesh3d(marker_mesh),
        MeshMaterial3d(marker_material),
        Visibility::from(marker_visibility),
        Marker(label.to_string())
    )).set_parent(parent);
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

pub(crate) fn despawn_all_markers(
    commands: &mut Commands,
    query_markers: &Query<(Entity, &C3dMarkers)>,
) {
    for (entity, _) in query_markers.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub(crate) fn despawn_all_markers_event(
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
