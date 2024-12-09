use crate::*;

#[derive(Component, Clone, PartialEq)]
/// This represents a vector. It contains the labels of the points that are joined. First point is the origin, second point is the vector, third parameter is the scale.
pub struct Vector(pub Marker, pub Marker, pub f64);

#[derive(Event)]
/// VectorEvent contains the events related to the vectors.
pub enum VectorEvent {
    HideAllVectorsEvent,
    ShowAllVectorsEvent,
    HideVectorEvent(Vector),
    ShowVectorEvent(Vector),    
}

pub(crate) fn spawn_vectors_in_config(
    current_config: &str,
    config_file: &ConfigFile,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
){
    if config_file.get_config(current_config).is_some(){
        if let Some(vectors) = config_file.get_config(current_config).unwrap().get_vectors(){
            for (point, vector) in vectors {
                let default_cylinder_height = 1.0;
                let mut cone_mesh = Mesh::from(Cone {
                    radius: 0.05,
                    height: 0.2,
                });
                let mut cylinder_mesh = Mesh::from(Cylinder::new(
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
                
                cylinder_mesh.merge(&cone_mesh);

                commands.spawn((
                    Mesh3d(meshes.add(cylinder_mesh)),
                    MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb_u8(255, 220, 0),
                        ..default()})),
                    Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
                    Vector(Marker(point.clone()), Marker(vector.0.clone()), vector.1.clone())));
                commands.spawn((
                    Mesh3d(meshes.add(cone_mesh)),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb_u8(255, 220, 0),
                        ..default()
                    })),
                    Transform::from_translation(Vec3::new(0.0, vector.1 as f32, 0.0)),
                    Vector(Marker(point.clone()), Marker(vector.0.clone()), vector.1.clone())));
            }
        }
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
                        let scale = Vec3::new(1.0, length, 1.0);
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

pub(crate) fn despawn_all_vectors(
    commands: &mut Commands,
    query_vectors: &Query<(Entity, &Vector)>,
) {
    for (entity, _) in query_vectors.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
