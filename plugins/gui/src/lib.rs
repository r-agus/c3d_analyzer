use bevy::{prelude::*, window::PrimaryWindow};

use bevy_egui::{egui::{self}, EguiContext, EguiPlugin};
use bevy_inspector_egui::{bevy_inspector::hierarchy::SelectedEntities, DefaultInspectorConfigPlugin};

use control_plugin::*;

pub struct GUIPlugin;

impl Plugin for GUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_plugins(DefaultInspectorConfigPlugin)
            .add_systems(Update,gui);
    }
}

fn gui(world: &mut World, 
        mut selected_entities: Local<SelectedEntities>,
    ) {
    let mut egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
        .clone();
    
    let hierarchy_enabled;
    let timeline_enabled ;
    {
        let gui_sides = world.get_resource_ref::<GuiSidesEnabled>().unwrap();
        hierarchy_enabled = gui_sides.hierarchy_inspector;
        timeline_enabled = gui_sides.timeline;
    }

    if hierarchy_enabled{
    // Inspector
    // ui.collapsing(heading, add_contents): interesting for the points
        egui::SidePanel::left("hierarchy")
            .default_width(200.0)
            .show(egui_context.get_mut(), |ui| {
                egui::ScrollArea::both().show(ui, |ui| {
                    ui.heading("Hierarchy");

                    bevy_inspector_egui::bevy_inspector::hierarchy::hierarchy_ui(
                        world,
                        ui,
                        &mut selected_entities,
                    );

                    ui.label("Press escape to toggle UI");
                    ui.allocate_space(ui.available_size());
                });
            });

        egui::SidePanel::right("inspector")
            .default_width(250.0)
            .show(egui_context.get_mut(), |ui| {
                egui::ScrollArea::both().show(ui, |ui| {
                    ui.heading("Inspector");

                    match selected_entities.as_slice() {
                        &[entity] => {
                            bevy_inspector_egui::bevy_inspector::ui_for_entity(world, entity, ui);
                        }
                        entities => {
                            bevy_inspector_egui::bevy_inspector::ui_for_entities_shared_components(
                                world, entities, ui,
                            );
                        }
                    }

                    ui.allocate_space(ui.available_size());
                });
        });
    }

    let mut app_state = world.get_resource_mut::<AppState>().unwrap();
    let mut frame  = app_state.frame;
    let mut path = app_state.path.clone();
    let num_frames = match app_state.num_frames {
        0 => 1,
        _ => app_state.num_frames,
    };

    // Timeline
    if timeline_enabled {
        egui::TopBottomPanel::bottom("Timeline").show(egui_context.get_mut(), |ui| {

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

        // FPS window
        // egui::Window::new("FPS")
        //     .show(egui_context.get_mut(), |ui| {
        //         ui.label(format!("{:.2}", app_state.fixed_frame_rate.unwrap_or(0.0)));
        //         ui.label(format!("{:.2}", app_state.frame_rate.unwrap_or(0.0)));
        //     });


        if app_state.frame != frame {
            app_state.frame = frame;
            app_state.render_frame = true;
        }
    }
}