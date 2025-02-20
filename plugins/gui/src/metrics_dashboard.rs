use std::collections::HashMap;

use crate::*;
use egui_plot::{Line, Plot};

#[derive(Resource, Default)]
pub(crate) struct Graphs{
    graphs: HashMap<String, Graph>,
    empty_graphs: HashMap<String, XYZ>,
} 

struct Graph {
    primary_plot: Vec<f64>,
    secondary_plot: Vec<f64>, 
    scale: Scale,
}

#[derive(Component)]
pub(crate) struct MarkersWindow;

enum Scale {
    Time,
    Frames,    
}

#[derive(Event)]
pub(crate) enum GraphEvent {
    AddGraph(String, XYZ),
    RemoveGraph(String),
    RestartGraphs,
    CreateMarkersWindow,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum XYZ {
    X = 0,
    Y = 1,
    Z = 2,
}

impl Graphs {
    fn new() -> Self {
        Graphs {
            graphs: HashMap::new(),
            empty_graphs: HashMap::new(),
        }
    }
    fn add_graph(&mut self, marker: String, primary: Vec<f64>) {
        self.graphs.insert(marker, Graph::new(primary));
    }
    fn add_empty_graph(&mut self, marker: String, xyz: XYZ) {
        self.empty_graphs.insert(marker, xyz);
    }
    fn remove_graph(&mut self, marker: &str) {
        self.graphs.remove(marker);
    }
    fn restart_graphs(&mut self) {
        self.graphs.iter_mut().for_each(|(_, graph)| graph.restart_secondary_plot());
    }
    fn set_time_scale(&mut self) {
        self.graphs.iter_mut().for_each(|(_, graph)| graph.scale = Scale::Time);
    }
    fn set_frame_scale(&mut self) {
        self.graphs.iter_mut().for_each(|(_, graph)| graph.scale = Scale::Frames);
    }
    fn set_scale(&mut self, scale: Scale) {
        self.graphs.iter_mut().for_each(|(_, graph)| graph.scale = match scale {
            Scale::Time => Scale::Time,
            Scale::Frames => Scale::Frames,
        });
    }
}

impl Graph{
    fn new(primary: Vec<f64>) -> Self {
        Graph {
            primary_plot: primary,
            secondary_plot: Vec::new(),
            scale: Scale::Frames,
        }
    }
    fn add_primary_plot(&mut self, value: Vec<f64>,){
        self.primary_plot = value;
    }
    fn add_secondary_plot(&mut self, value: Vec<f64>,){
        self.secondary_plot = value;
    }
    fn restart_secondary_plot(&mut self){
        self.secondary_plot.clear();
    }
    fn get_primary_plot(&self) -> Vec<[f64; 2]> {
        self.primary_plot.iter().enumerate().map(|(i, &v)| [i as f64, v]).collect()
    }
    fn get_secondary_plot(&self) -> Vec<[f64; 2]> {
        self.secondary_plot.iter().enumerate().map(|(i, &v)| [i as f64, v]).collect()
    }
}

impl MarkersWindow {
    fn new() -> Self {
        MarkersWindow
    }
    pub(crate) fn draw_floating_window(
        mut ctx: EguiContexts,
        mut commands: Commands,
        mut graphs: ResMut<Graphs>,
        mut trace_event: EventWriter<TraceEvent>,
        query_markers: Query<&Marker>,
        query_traces:  Query<&Trace>,
        query_windows: Query<(Entity, &Self)>,
        config_state: Res<ConfigState>,
        config_assets: Res<Assets<ConfigC3dAsset>>,
    ) {
        let config_state = config_assets.get(&config_state.handle);
        let traces = query_traces.iter().map(|trace| trace.0.clone()).collect::<Vec<String>>();
        for (entity, _) in query_windows.iter() {
            let ctx = ctx.ctx_mut();
            let mut open = true;

            egui::Window::new("Markers")
                .scroll([false, true])
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.label("Select a marker to add a graph");
                    ui.separator();
                    let mut markers = query_markers.iter().map(|marker| marker.0.clone()).collect::<Vec<String>>();
                    markers.sort();
                    markers.dedup();

                    let mut represented_points = Vec::new();
                    
                    if let Some(config_state) = config_state {
                        config_state.config.get_config_map().iter().for_each(|(key, value)| {
                            ui.collapsing(key, |ui| {
                                let binding = vec![];
                                let mut markers_in_config = value.get_visible_points().unwrap_or(&binding).clone();
                                markers_in_config.sort();
                                markers_in_config.dedup();
                                represented_points = draw_childs(ui, &mut graphs, &mut trace_event, &markers_in_config, &traces)
                            });
                        });
                    }

                    markers.retain(|marker| !represented_points.contains(marker));

                    ui.collapsing("Not in config", |ui| {
                        draw_childs(ui, &mut graphs, &mut trace_event, &markers, &traces);
                    });
                });
            if !open {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn draw_childs (
    ui: &mut egui::Ui,
    graphs: &mut ResMut<Graphs>,
    trace_event: &mut EventWriter<TraceEvent>,
    markers: &Vec<String>,
    traces: &Vec<String>,
) -> Vec<String> {
    let mut represented_points = Vec::new();
    for marker in markers {
        ui.horizontal(|ui| {
            let (text, contained) = if traces.contains(&marker) {
                ("Remove Trace", true)    
            } else {
                ("Trace", false)
            };
            if ui.button(text).clicked() {
                match contained {
                    true =>  trace_event.send(TraceEvent::DespawnTraceEvent(marker.clone())),
                    false => trace_event.send(TraceEvent::AddTraceEvent(marker.clone())),
                };
            }
            ui.collapsing(marker.clone(), |ui| {
                if ui.button("Plot X").clicked() {
                    graphs.add_empty_graph(marker.to_string(), XYZ::X);
                }
                if ui.button("Plot Y").clicked() {
                    graphs.add_empty_graph(marker.to_string(), XYZ::Y);
                }
                if ui.button("Plot Z").clicked() {
                    graphs.add_empty_graph(marker.to_string(), XYZ::Z);
                }
            });
        });
        represented_points.push(marker.clone());
    }
    represented_points
}

impl XYZ {
    fn _to_string(&self) -> String {
        match self {
            XYZ::X => "::x".to_string(),
            XYZ::Y => "::y".to_string(),
            XYZ::Z => "::z".to_string(),
        }
    }
    fn to_str(&self) -> &str {
        match self {
            XYZ::X => "::x",
            XYZ::Y => "::y",
            XYZ::Z => "::z",
        }
    }
}

pub(crate) fn fill_empty_graphs(
    mut event_writer: EventWriter<GraphEvent>,
    mut graphs: ResMut<Graphs>,
){
    for (marker, graph) in graphs.empty_graphs.iter() {
        event_writer.send(GraphEvent::AddGraph(marker.to_string(), *graph));
    }
    graphs.empty_graphs.clear();
}

pub(crate) fn graph_event_orchestrator(
    mut event_reader: EventReader<GraphEvent>,
    mut graphs: ResMut<Graphs>,
    mut commands: Commands,
    mut ctx: EguiContexts,
    c3d_state: Res<bevy_c3d_mod::C3dState>,
    c3d_assets: Res<Assets<bevy_c3d_mod::C3dAsset>>,
    query_markers: Query<(&Marker, &Transform)>,
    query_windows: Query<(Entity, &MarkersWindow)>,
){
    if let Some(event) = event_reader.read().last() {
        match event {
            GraphEvent::AddGraph(marker, idx) => {
                let marker_position = get_marker_position_on_all_frames(marker, &c3d_state, &c3d_assets, &query_markers)
                    .map_or(vec![0.0], |vectores| vectores.iter().map(|v| v[*idx as usize] as f64).collect());
                graphs.add_graph(marker.to_string() + idx.to_str(), marker_position);
            }
            GraphEvent::RemoveGraph(marker) => {
                graphs.remove_graph(marker);
            }
            GraphEvent::RestartGraphs => {
                graphs.restart_graphs();
            }
            GraphEvent::CreateMarkersWindow => {
                let ctx = ctx.ctx_mut();
                if query_windows.iter().count() == 0 {
                    commands.spawn(MarkersWindow::new());
                }
            }
        }
    }
}

pub(crate) fn fill_graphs(
    mut graphs: ResMut<Graphs>,
    state: Res<AppState>,
){
    let frame = state.frame;
    
    graphs
        .graphs
        .iter_mut()
        .for_each(|(_, graph)| {
            let secondary_plot = if graph.primary_plot.len() >= frame {
                graph.primary_plot[0..frame].to_vec()
            } else {
                Vec::new()
            };
            graph.add_secondary_plot(secondary_plot);
        });
}

pub(crate) fn represent_graphs(
    mut graphs: ResMut<Graphs>,
    mut ctx: EguiContexts,
    mut commands: Commands,
    query_markers: Query<&Marker>
){
    let ctx  = ctx.ctx_mut();
    let mut removed_graphs = Vec::new();
    // let markers = query_markers.iter().map(|marker| marker.0.clone()).collect::<Vec<String>>();

    egui::SidePanel::right("Graphs")
        .show(ctx, |ui| {
            if ui.button("Add graph").clicked() {
                commands.spawn(MarkersWindow::new());
            }
            ui.separator();
            ui.collapsing("Settings", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Scale:");
                    if ui.button("Time").clicked() {
                        graphs.set_time_scale();
                    }
                    if ui.button("Frames").clicked() {
                        graphs.set_frame_scale();
                    }
                });
            });
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut keys = graphs.graphs.keys().cloned().collect::<Vec<String>>();
                keys.sort();
                for (_i, marker) in keys.iter().enumerate() {
                    ui.collapsing(marker, |ui|{
                        if ui.button("Remove").clicked() {
                            removed_graphs.push(marker.clone());
                        }
                        
                        let new_plot = || {
                            Plot::new(marker)
                                .allow_scroll(false)
                                .view_aspect(2.0)
                                .auto_bounds([true, true].into())
                        };
                        let binding = Graph::new(vec![]);
                        let graph = graphs.graphs.get(marker).unwrap_or(&binding);
                        let principal_line = Line::new(graph.get_primary_plot())
                            .color(egui::Color32::from_rgb(255, 0, 0));
                        let secondary_line = Line::new(graph.get_secondary_plot())
                            .color(egui::Color32::from_rgb(0, 255, 0));
                        let plot = match graph.scale {
                            Scale::Time => new_plot().x_axis_label("Time"),
                            Scale::Frames => new_plot().x_axis_label("Frames"),
                        };
                        plot.show(ui, |ui| {
                            ui.line(principal_line);
                            ui.line(secondary_line);
                        });
                    });
                }
            });
        });
    
    for marker in removed_graphs {
        graphs.remove_graph(&marker);
    }
}