use bevy::prelude::*;

use bevy_egui::{egui::{self}, EguiContexts, EguiPlugin};
use bevy_metrics_dashboard::{metrics::{describe_gauge, gauge}, DashboardPlugin, DashboardWindow, RegistryPlugin};

use control_plugin::*;

pub struct GUIPlugin;

impl Plugin for GUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_plugins(RegistryPlugin::default())
            .add_plugins(DashboardPlugin)
            .add_systems(Startup, create_dashboard)
            .add_systems(Update,gui);
    }
}

fn create_dashboard(
    mut commands: Commands,
) {
    commands.spawn(DashboardWindow::new("Graphs"));
}

fn _describe_graphs(
    // markers: Query<(&C3dMarkers, &Children)>,
    marker: Query<&Marker>
){
    println!("Describing graphs");
    println!("Markers: {:?}", marker.iter().count());
    for m in marker.iter() {
        describe_gauge!("Test Gauge", m.0.clone());
        println!("Describing gauge: {}", m.0);
    }
}

fn gui(
    mut egui_context: EguiContexts,
    mut app_state: ResMut<AppState>,
    gui_sides: ResMut<GuiSidesEnabled>,
    markers_query: Query<(&Marker, &Transform)>,
) {
    let timeline_enabled ;
    let _graphs_enabled;
    {
        timeline_enabled = gui_sides.timeline;
        _graphs_enabled = gui_sides.graphs;
    }
    let mut frame  = app_state.frame;
    let mut path = app_state.c3d_path.clone();
    let num_frames = match app_state.num_frames {
        0 => 1,
        _ => app_state.num_frames,
    };

    // Timeline
    if timeline_enabled {
        egui::TopBottomPanel::bottom("Timeline").show(egui_context.ctx_mut(), |ui| {
            let frame_slider = egui::Slider::new(&mut frame, 0..=(num_frames - 1)).show_value(true);
            let half_width = ui.available_width() * 0.5; 

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
                        ui.add(frame_slider
                            .handle_shape(egui::style::HandleShape::Rect{ aspect_ratio: 0.1 })
                        );
                    });
                });

                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Play:");
                        ui.checkbox(&mut app_state.play, "");
                    });
                });

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
                }

                // });
            });
        });

        for (m,t) in markers_query.iter() {
            let pos = t.translation;
            gauge!(m.0.clone() + "::x").set(pos[0]);  // TODO: group by config
            gauge!(m.0.clone() + "::y").set(pos[1]);
            gauge!(m.0.clone() + "::z").set(pos[2]);
        }

        if app_state.frame != frame {
            app_state.frame = frame;
            app_state.render_frame = true;
        }
    }
}