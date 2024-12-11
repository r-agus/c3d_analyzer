mod milestones;
mod metrics_dashboard;

use bevy::prelude::*;

use bevy_egui::{egui::{self, Sense}, EguiContexts, EguiPlugin};

// #[cfg(not(target_arch = "wasm32"))]
// use bevy_metrics_dashboard::{metrics::{describe_gauge, gauge}, DashboardPlugin, DashboardWindow, RegistryPlugin};

use config_plugin::{ConfigC3dAsset, ConfigState};
use control_plugin::*;
use egui_double_slider::DoubleSlider;
use milestones::{milestones_event_orchestrator, update_milestone_board, Milestones};
use metrics_dashboard::*;
use vectors::*;
use markers::*;
use traces::*;

pub struct GUIPlugin;

impl Plugin for GUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            // .add_plugins(RegistryPlugin::default())
            // .add_plugins(DashboardPlugin)
            .add_systems(Startup, setup)
            .add_systems(Update,
                    (gui, 
                        fill_graphs, represent_graphs
                    // DashboardWindow::draw_all.run_if(|state: Res<GuiSidesEnabled>| -> bool { state.graphs } )
                    ).chain())
            .add_systems(Update, (milestones_event_orchestrator, graph_event_orchestrator, fill_empty_graphs, MarkersWindow::draw_all))
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

fn gui(
    mut trace_event: EventWriter<TraceEvent>,
    mut vector_event: EventWriter<VectorEvent>,
    mut egui_context: EguiContexts,
    mut app_state: ResMut<AppState>,
    mut milestones: ResMut<Milestones>,
    gui_sides: ResMut<GuiSidesEnabled>,
    config_state: Res<ConfigState>,
    config_assets: Res<Assets<ConfigC3dAsset>>,
    // markers_query: Query<(&Marker, &Transform)>,
    vectors_query: Query<(&Vector, &Visibility)>,
) {
    let timeline_enabled;
    let graphs_enabled;
    {
        timeline_enabled = gui_sides.timeline;
        graphs_enabled = gui_sides.graphs;
    }
    let mut slider_frame  = app_state.frame;
    let mut milestone_frame = app_state.frame;
    let mut path = app_state.c3d_path.clone();
    let num_frames = match app_state.num_frames {
        0 => 1,
        _ => app_state.num_frames,
    };

    // Timeline
    if timeline_enabled {
        egui::TopBottomPanel::bottom("Timeline").show(egui_context.ctx_mut(), |ui| {
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
                    
                    let milestones = milestones.as_mut();
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
                            let config_state = config_assets.get(&config_state.handle);
                            if let Some(config_state) = config_state {
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
        });

        // #[cfg(not(target_arch = "wasm32"))]
        // for (m,t) in markers_query.iter() {
        //     let pos = t.translation;
        //     gauge!(m.0.clone() + "::x").set(pos[0]);  // TODO: group by config
        //     gauge!(m.0.clone() + "::y").set(pos[1]);
        //     gauge!(m.0.clone() + "::z").set(pos[2]);
        // }

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
    if graphs_enabled {
        
    }
}