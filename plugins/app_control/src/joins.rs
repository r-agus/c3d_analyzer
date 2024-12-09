use crate::*;

#[derive(Component)]
/// This represents the joins between the points in the C3D file. It contains the labels of the points that are joined.
pub struct Join(pub(crate) String, pub(crate) String);

#[derive(Event)]
pub enum JoinEvent {
    DespawnAllJoinsEvent,
    DespawnJoinEvent(String, String),
}

pub(crate) fn spawn_joins_in_config(
    current_config: &str,
    config_file: &ConfigFile,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
){
    if config_file.get_config(current_config).is_some(){
        if let Some(joins) = config_file.get_config(current_config).unwrap().get_joins(){
            for join in joins {
                for i in 0..join.len() - 1 {
                    let line_thickness = config_file.get_line_thickness(&join[i], &join[i+1], &current_config).unwrap_or(0.01) as f32;
                    let line_color = config_file.get_join_color(&join[i], &join[i+1], &current_config).unwrap_or(vec![0, 255, 0]);
                    let join_mesh =  meshes.add(
                            Cylinder::new(
                                if line_thickness > 0.01 { line_thickness * 0.01 } else { 0.01 },
                                1.0)
                        );
                    let join_material = materials.add(StandardMaterial {
                        base_color: if line_color.len() == 3 {
                                        Color::srgb_u8(line_color[0], line_color[1],line_color[2])
                                    } else if line_color.len() == 4 {
                                        Color::srgba_u8(line_color[0], line_color[1], line_color[2], line_color[3])
                                    } else{
                                        Color::srgb_u8(0, 127, 0)
                                    },
                        ..default()
                    });

                    commands.spawn((
                        Mesh3d(join_mesh),
                        MeshMaterial3d(join_material),
                        Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
                        Join(join[i].clone(), join[i+1].clone())));
                }
            }
        }
    }
}

pub fn represent_joins(
    mut join_event: EventWriter<JoinEvent>,
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
                        join_event.send(JoinEvent::DespawnJoinEvent(join.0.clone(), join.1.clone()));
                    }
                }
            }      
        },
        None => {}
    }
}

pub fn joins_event_orchestrator(
    mut events: EventReader<JoinEvent>,
    mut commands: Commands,
    query_joins: Query<(Entity, &Join)>,
){
    if let Some(join_event) = events.read().last() {
        match join_event {
            JoinEvent::DespawnAllJoinsEvent => {
                despawn_all_joins(&mut commands, &query_joins);
            }
            JoinEvent::DespawnJoinEvent(point1, point2) => {
                delete_join_event(&mut commands, &query_joins, point1, point2);
            }
        }
    }
}

fn delete_join_event(
    commands: &mut Commands,
    query_joins: &Query<(Entity, &Join)>,
    point1: &str,
    point2: &str,
) {
    for (entity, join) in query_joins.iter() {
        if join.0 == point1 && join.1 == point2 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub(crate) fn despawn_all_joins(
    commands: &mut Commands,
    query_joins: &Query<(Entity, &Join)>,
) {
    for (entity, _) in query_joins.iter() {
        commands.entity(entity).despawn_recursive();
    }
}