use std::{collections::HashMap, ops::Deref};

use crate::*;
use bevy::{core_pipeline::core_3d::graph, state::commands};
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
enum XYZ {
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
    pub(crate) fn draw_all(
        mut ctx: EguiContexts,
        query_windows: Query<&Self>,
        query_markers: Query<&Marker>,
        mut graphs: ResMut<Graphs>,
    ) {
        for _ in query_windows.iter() {
            let ctx = ctx.ctx_mut();
            egui::Window::new("Markers").show(ctx, |ui| {
                ui.label("Select a marker to add a graph");
                ui.separator();
                for marker in &query_markers {
                    ui.collapsing(marker.0.clone(), |ui| {
                        if ui.button("x").clicked() {
                            graphs.add_empty_graph(marker.0.to_string(), XYZ::X);
                        }
                        if ui.button("y").clicked() {
                            graphs.add_empty_graph(marker.0.to_string(), XYZ::Y);
                        }
                        if ui.button("z").clicked() {
                            graphs.add_empty_graph(marker.0.to_string(), XYZ::Z);
                        }
                    });
                }
            });
        }
    }
}

impl XYZ {
    fn to_string(&self) -> String {
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
    graphs: ResMut<Graphs>,
){
    for (marker, graph) in graphs.empty_graphs.iter() {
        event_writer.send(GraphEvent::AddGraph(marker.to_string(), *graph));
    }
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
            let secondary_plot = graph.primary_plot[0..frame].to_vec();
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
    let markers = query_markers.iter().map(|marker| marker.0.clone()).collect::<Vec<String>>();

    egui::SidePanel::right("Graphs")
        .show(ctx, |ui| {
            if ui.button("Add graph").clicked() {
                commands.spawn(MarkersWindow::new());
            }
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (i, (marker, graph)) in graphs.graphs.iter_mut().enumerate() {
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