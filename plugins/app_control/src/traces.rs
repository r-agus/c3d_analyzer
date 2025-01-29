use crate::*;

#[derive(Component)]
/// This is the trace of a point along the frames
pub struct Trace(pub String);


#[derive(Clone, Default, Debug)]
/// TraceInfo contains the information of the traces to be represented.
/// A trace is the representation of a point along the frames in a given range, with no time information.
/// start_frame: The frame where the trace starts.
/// end_frame: The frame where the trace ends.
/// points: The points that are part of the trace.
pub struct TraceInfo {
    pub start_frame: f32,
    pub end_frame: f32,
    pub points: Vec<String>, // Replace by Box ??
}

#[derive(Event)]
/// TraceEvent contains the events related to the traces.
pub enum TraceEvent {
    AddTraceEvent(String),
    UpdateTraceEvent,
    DespawnTraceEvent(String),
    DespawnAllTracesEvent,
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
            TraceEvent::AddTraceEvent(trace) => {
                if !state.traces.is_trace_added(trace.clone()) {
                    state.traces.add_point(trace.clone());
                }
                despawn_all_traces(&mut commands, query_delete_trace);
                represent_traces(&mut commands, &mut meshes, &mut materials, &state, &c3d_state, &c3d_assets, &query_positions);
            }
            TraceEvent::UpdateTraceEvent => {
                despawn_all_traces(&mut commands, query_delete_trace);
                represent_traces(&mut commands, &mut meshes, &mut materials, &state, &c3d_state, &c3d_assets, &query_positions);
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

fn represent_traces(
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
                        Mesh3d(meshes.add(
                                Sphere::new(0.005).mesh()
                            )),
                        MeshMaterial3d(materials.add(StandardMaterial {
                                base_color: Color::srgb_u8(49, 0, 69),
                                ..default()
                            })),
                        Transform::from_translation(position),    
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