use std::{collections::HashMap, ops::DerefMut, usize};

use bevy::{ecs::label, prelude::{ResMut, Resource}, ui};
use bevy_egui::{egui::{Response, Ui}, EguiContexts};
use egui_plot::{Legend, Line, Plot, PlotResponse};

#[derive(Resource, Default)]
pub struct Milestones {
    milestones: HashMap<usize, MilestoneType>,  // Frame, MilestoneType
    dirty: bool, // For changes
}

enum MilestoneType {
    UserGenerated,
    FromC3d,
}

impl Milestones {
    pub fn default(&mut self) {
        self.milestones = HashMap::new();
        self.dirty = true;
    }

    fn add_milestone(&mut self, frame: usize, milestone_type: MilestoneType) {
        self.milestones.insert(frame, milestone_type);
        self.dirty = true;
    }

    fn clear_dirty(&mut self){
        self.dirty = false;
    }

    pub fn add_user_generated(&mut self, frame: usize) {
        self.add_milestone(frame, MilestoneType::UserGenerated);
    }
    
    pub fn add_from_c3d(&mut self, frame: usize) {
        self.add_milestone(frame, MilestoneType::FromC3d);
    }
}

// Update board
pub(crate) fn update_milestone_board(milestones: &mut Milestones, num_frames: usize, ui: &mut Ui) {
    // if milestones.dirty {  // Can we avoid redrawing every frame?
        let points: Vec<_> = milestones
            .milestones
            .keys()
            .collect();

        let new_plot = egui_plot::Plot::new("milestones")
            .allow_zoom(false)
            .allow_scroll(false)
            .allow_drag(false)
            .allow_boxed_zoom(false)
            .show_grid([false, false])
            .show_axes([false, false])
            .center_x_axis(false)
            // .show_background(false)  // Activate this when ready
            .height(15.)
            .include_x(0.0)
            .include_x(num_frames as f32)
            .width(ui.available_width() / 1.58);
        
        ui.horizontal(|ui|{
            ui.label("Events:");
            new_plot.show(ui, |plot_ui| {
                plot_ui.line(Line::new(vec![
                    [10.0, -1.0],
                    [10.0, 1.0]
                ]));
                for &p in points {
                    plot_ui.line(Line::new(vec![
                        [p as f64, 1.0],
                        [p as f64, -1.0]
                    ]));
                }
            }).response;
        });
        milestones.clear_dirty();
    //  }
}