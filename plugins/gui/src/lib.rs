use bevy::{input::common_conditions::input_toggle_active, prelude::*, window::PrimaryWindow};

use bevy_c3d_mod::{C3dAsset, C3dState};
use bevy_egui::{egui, EguiContext, EguiContexts, EguiPlugin};
use bevy_inspector_egui::{bevy_inspector::hierarchy::SelectedEntities, DefaultInspectorConfigPlugin};

use control_plugin::*;

pub struct GUIPlugin;

impl Plugin for GUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_plugins(DefaultInspectorConfigPlugin)
            .add_systems(
                Update,
                inspector_ui.run_if(input_toggle_active(true, KeyCode::Escape)),
            )
            .add_systems(Update, timeline);

    }
}

fn timeline(
    mut state: ResMut<AppState>,
    mut egui_context: EguiContexts,
    query_points: Query<(&C3dMarkers, &Children)>,          // Points and their children (Markers)
    query_markers: Query<(&mut Transform, &Marker)>,
    c3d_state: Res<C3dState>,
    c3d_assets: Res<Assets<C3dAsset>>,
) {
    let mut frame = state.frame;
    let num_frames = state.num_frames;
    let mut play = state.play;
    let mut path = state.path.clone();

    egui::TopBottomPanel::bottom("Timeline").show(egui_context.ctx_mut(),|ui| {
        ui.horizontal(|ui| {
            ui.label("Frame:");
            ui.add(egui::Slider::new(&mut frame, 0..=(num_frames - 1)));
        });

        ui.horizontal(|ui| {
            ui.label("Path:");
            ui.text_edit_singleline(&mut path);
        });

        ui.horizontal(|ui| {
            ui.label("Play:");
            ui.checkbox(&mut play, "");
        });
    });

    state.play = play;
    state.path = path;

    if state.frame != frame {
        state.frame = frame;
        represent_points(state, query_points, query_markers, c3d_state, c3d_assets);
    }
}


fn inspector_ui(world: &mut World, 
        mut selected_entities: Local<SelectedEntities>,
    ) {
    let mut egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
        .clone();

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