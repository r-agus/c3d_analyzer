mod milestones;
mod metrics_dashboard;

use bevy::prelude::*;

use bevy_egui::{egui::{self, Sense, Ui}, EguiContexts, EguiPlugin};

use config_plugin::{ConfigC3dAsset, ConfigState};
use control_plugin::*;
use egui_double_slider::DoubleSlider;
use egui_dock::{DockState, NodeIndex, SurfaceIndex};

use milestones::{milestones_event_orchestrator, update_milestone_board, Milestones};
use metrics_dashboard::*;
use vectors::*;
use markers::*;
use traces::*;

enum AppTab {
    Scene,
    Graphs,
    Timeline,
}

#[derive(Resource)]
struct UiState {
    app_state: &'static mut AppState,
    milestones: &'static mut Milestones,
    graphs: &'static mut Graphs,
    trace_event_writer: &'static mut EventWriter<'static, TraceEvent>,
    vector_event_writer: &'static mut EventWriter<'static, VectorEvent>,
    config_state: &'static ConfigState,
    config_assets: &'static ConfigC3dAsset,
    graph_event: &'static mut EventWriter<'static, GraphEvent>,
    vectors_query: &'static Query<'static, 'static, (&'static Vector, &'static Visibility)>,

    dock: DockState<AppTab>,
}

impl UiState {
    fn new(
        app_state: &'static mut AppState,
        milestones: &'static mut Milestones,
        graphs: &'static mut Graphs,
        trace_event_writer: &'static mut EventWriter<'static, TraceEvent>,
        vector_event_writer: &'static mut EventWriter<'static, VectorEvent>,
        config_state: &'static ConfigState,
        config_assets: &'static ConfigC3dAsset,
        graph_event: &'static mut EventWriter<'static, GraphEvent>,
        vectors_query: &'static Query<'static, 'static, (&'static Vector, &'static Visibility)>,
    ) -> UiState {
        Self {
            app_state,
            milestones,
            graphs,
            trace_event_writer,
            vector_event_writer,
            config_state,
            config_assets,
            graph_event,
            vectors_query,
        
            dock: DockState::new(vec![AppTab::Scene]),
        }
    }

    fn ui(&mut self, ctx: &mut EguiContexts) {
        // let mut added_tabs = Vec::new();
        // let mut tab_viewer = TabViewer{

        // };
    }
}


impl<'a> egui_dock::TabViewer for UiState {
    type Tab = AppTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui_dock::egui::WidgetText {
        match tab {
            AppTab::Scene => "Scene".into(),
            AppTab::Graphs => "Graphs".into(),
            AppTab::Timeline => "Timeline".into(),
        }
    }

    fn ui(&mut self, ui: &mut egui_dock::egui::Ui, tab: &mut Self::Tab) {
        match tab {
            AppTab::Scene => {
                let rect = ui.available_rect_before_wrap();
                ui.allocate_rect(rect, Sense::hover());
            }
            AppTab::Graphs => {
                represent_graphs(self.graphs, ui, self.graph_event);
            }
            AppTab::Timeline => {
                represent_timeline(
                    self.trace_event_writer,
                    self.vector_event_writer,
                    ui,
                    self.app_state,
                    self.milestones,
                    self.config_assets,
                    self.vectors_query,
                );
            },
        }
    }
    
    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool { true }
    
    fn on_add(&mut self, _surface: egui_dock::SurfaceIndex, _node: NodeIndex) {}
    
    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool { true }
    
    fn clear_background(&self, _tab: &Self::Tab) -> bool { true }
    
    fn scroll_bars(&self, _tab: &Self::Tab) -> [bool; 2] { [true, true] }
}

pub struct GUIPlugin;

impl Plugin for GUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            // .add_plugins(RegistryPlugin::default())
            // .add_plugins(DashboardPlugin)
            .add_systems(Startup, setup)
            .add_systems(Update,fill_graphs)
            .add_systems(Update, (milestones_event_orchestrator, graph_event_orchestrator, fill_empty_graphs, MarkersWindow::draw_floating_window))
            .init_resource::<Graphs>()
            .init_resource::<Milestones>()
            .add_event::<GraphEvent>();
    }
}

fn setup(
    // mut commands: Commands,
    mut milestones: ResMut<Milestones>,
) {
    // commands.spawn(DashboardWindow::new("Graphs"));
    milestones.default();
}

fn gui(
    mut egui_contexts: EguiContexts,
    ui_state: Res<UiState>,
) {
    let ctx = egui_contexts.ctx_mut();
    
}

fn _describe_graphs(
    // markers: Query<(&C3dMarkers, &Children)>,
    marker: Query<&Marker>
){
    println!("Describing graphs");
    println!("Markers: {:?}", marker.iter().count());
    for m in marker.iter() {
        // describe_gauge!("Test Gauge", m.0.clone());
        println!("Describing gauge: {}", m.0);
    }
}

fn represent_timeline(
    trace_event: &mut EventWriter<TraceEvent>,
    vector_event: &mut EventWriter<VectorEvent>,
    ui: &mut Ui,
    app_state: &mut AppState,
    milestones: &mut Milestones,
    config_assets: &ConfigC3dAsset,
    vectors_query: &Query<(&Vector, &Visibility)>,
) {
    
    let mut slider_frame  = app_state.frame;
    let mut milestone_frame = app_state.frame;
    let mut path = app_state.c3d_path.clone();
    let num_frames = match app_state.num_frames {
        0 => 1,
        _ => app_state.num_frames,
    };
    let frame_slider = egui::Slider::new(&mut slider_frame, 0..=(num_frames - 1)).show_value(false);
    let half_width = ui.available_width() * 0.5; 
    let mut slider_width = 0.0;

    ui.spacing_mut().slider_width = half_width;
    ui.spacing_mut().text_edit_width = half_width * 0.35;
    ui.spacing_mut().tooltip_width = half_width * 0.5;

    ui.horizontal(|ui| {
        // ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {  // This might be an egui bug
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let path_label = ui.label("Path: ");
                ui.text_edit_singleline(&mut path)
                    .labelled_by(path_label.id)
                    .on_hover_text(path);
            });
        });

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Frame:");
                let frame_slider_resp = ui.add(frame_slider
                    .handle_shape(egui::style::HandleShape::Rect{ aspect_ratio: 0.1 })
                );
                slider_width = frame_slider_resp.rect.width();
            });
            
            let milestones = &milestones;
            update_milestone_board(milestones, slider_width, num_frames, ui);

            ui.horizontal(|ui|{
                let (start_frame, end_frame) = {
                    let traces = &mut app_state.traces;
                    (&mut traces.start_frame, &mut traces.end_frame)
                };
                let start_frame_copy = *start_frame;
                let end_frame_copy = *end_frame;
                ui.label("Traces:");
                ui.add(DoubleSlider::new(start_frame, end_frame, 0.0..=(num_frames - 1) as f32)
                    .separation_distance(1.0)
                    .width(half_width));

                if start_frame_copy as usize != *start_frame as usize || end_frame_copy as usize != *end_frame as usize {
                    trace_event.send(TraceEvent::UpdateTraceEvent);
                }
            });
        });

        ui.vertical(|ui| {
            ui.horizontal(|ui| { // TODO: Align this to the right
                ui.label(app_state.frame.to_string());
                if app_state.render_at_fixed_frame_rate {
                    ui.vertical(|ui| {  
                        ui.horizontal(|ui| {                 
                            ui.spacing_mut().slider_width = ui.available_width() * 0.7;

                            match app_state.frame_rate {
                                Some(c3d_frame_rate) => {
                                    let mut speed = if let Some(fixed_frame_rate) = app_state.fixed_frame_rate {fixed_frame_rate / c3d_frame_rate as f64} else {1.0};
                                    let speed_slider;
                                    speed_slider = egui::Slider::new(&mut speed, 0.1..=2.).fixed_decimals(1);
                                    ui.add(speed_slider);
                                    app_state.fixed_frame_rate = Some(c3d_frame_rate as f64 * speed);
                                },
                                None => {},                                
                            };
                        });
                    });
                } else {
                    ui.allocate_exact_size([1.0, ui.spacing().slider_rail_height].into(), Sense::hover());
                }
            });
            ui.horizontal(|ui| {
                // ⏮⏪⏩⏭
                let prev_milestone_button = ui.button("|◀").on_hover_text("Previous milestone");
                let add_milestone_button = ui.button("🔹").on_hover_text("Add milestone");
                let play_pause_button = if app_state.play 
                    {
                        ui.button("⏸").on_hover_text("Pause")
                    } else {
                        ui.button("▶").on_hover_text("Play")
                    };
                let next_milestone_button = ui.button("▶|").on_hover_text("Next milestone");
                ui.menu_button("Remove milestones", |ui| {
                    ui.label("Remove milestones");
                    let mut frames_to_remove = Vec::new();
                    for frame in milestones.get_milestones() {
                        if ui.button(format!("Frame {}", frame)).clicked() {
                            frames_to_remove.push(*frame);
                        }
                    }
                    for frame in frames_to_remove {
                        milestones.remove_milestone(frame);
                    }
                });
                let remove_user_milestones = ui.button("🔄").on_hover_text("Reset milestones");
                if prev_milestone_button.clicked() {
                    let prev = milestones.get_prev_milestone(app_state.frame);
                    milestone_frame = prev;
                }
                if next_milestone_button.clicked() {
                    let next = milestones.get_next_milestone(app_state.frame);
                    if next != 0 {
                        milestone_frame = next;
                    } else {
                        milestone_frame = num_frames - 2;
                    }
                }
                if add_milestone_button.clicked() {
                    milestones.add_user_generated(app_state.frame);
                }
                if play_pause_button.clicked() {
                    app_state.play = !app_state.play;
                }
                if remove_user_milestones.clicked() {
                    milestones.remove_user_generated_milestones();
                }
            });

            ui.horizontal(|ui| {
                ui.menu_button("Select configuration", |ui|{
                    ui.label("Select configuration");
                    let config_state = config_assets;
                    if let Some(config_state) = Some(config_state) {
                        for config_name in config_state.config.get_all_config_names() {
                            if ui.button(config_name.clone()).clicked() {
                                app_state.current_config = Some(config_name.clone());
                                app_state.change_config = true;
                            }
                        }
                    }
                });
                if ui.button("Remove all traces").on_hover_text("Remove all traces").clicked() {
                    trace_event.send(TraceEvent::DespawnAllTracesEvent);
                }
                ui.menu_button("Vectors", |ui| {
                    if ui.button("Hide all").clicked() {
                        vector_event.send(VectorEvent::HideAllVectorsEvent);
                    } 
                    if ui.button("Show all").clicked() {
                        vector_event.send(VectorEvent::ShowAllVectorsEvent);
                    }
                    
                    let mut vectors = vectors_query // for some reason query duplicates the vectors, so filter them out
                        .iter()
                        .collect::<Vec<_>>();
                    vectors.dedup();
                    for (vector, visibility) in vectors {
                        if ui.button(vector.0.0.clone()).clicked() {
                            if visibility == Visibility::Visible {
                                vector_event.send(VectorEvent::HideVectorEvent(vector.clone()));
                            } else {
                                vector_event.send(VectorEvent::ShowVectorEvent(vector.clone()));
                            }
                        }
                    }
                });
            });
        });
        // });
    });

    if app_state.frame != slider_frame {
        if slider_frame > 0 {
            app_state.frame = slider_frame - 1;
        } else {
            app_state.frame = 0;
        }
        app_state.render_frame = true;
    } else if app_state.frame != milestone_frame {
        if milestone_frame > 0 {
            app_state.frame = milestone_frame - 1;
        } else {
            app_state.frame = 0;
        }
        app_state.render_frame = true;
    }
}