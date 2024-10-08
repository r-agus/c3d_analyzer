use bevy::{prelude::*, window::PrimaryWindow};

use bevy_egui::{egui::{self, Layout}, EguiContext, EguiPlugin};
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
    
    let num_frames = match app_state.num_frames {
        0 => 1,
        _ => app_state.num_frames,
    };

    // Timeline
    if timeline_enabled {
        egui::TopBottomPanel::bottom("Timeline").show(egui_context.get_mut(), |ui| {

            let slider = egui::Slider::new(&mut frame, 0..=(num_frames - 1)).show_value(true);
            let half_width = ui.available_width() * 0.5; 
            
            ui.spacing_mut().slider_width = half_width;
            ui.spacing_mut().text_edit_width = half_width;

            // ui.allocate_ui_with_layout(ui.available_size(), Layout::, add_contents)
            ui.vertical_centered(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Frame:");
                    ui.add( slider);
                });
        
                ui.horizontal(|ui| {
                    ui.label("Path:");
                    ui.text_edit_singleline(&mut app_state.path);
                });
        
                ui.horizontal(|ui| {
                    ui.label("Play:");
                    ui.checkbox(&mut app_state.play, "");
                });
            });
            
        });
        
        if app_state.frame != frame {
            app_state.frame = frame;
            app_state.render_frame = true;
        }
    }
}