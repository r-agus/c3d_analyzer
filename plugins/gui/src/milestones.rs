use std::{collections::HashMap, usize};

use bevy::prelude::Resource;
use bevy_egui::egui::Ui;
use egui_plot::{Line, PlotBounds};

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

    pub(crate) fn add_user_generated(&mut self, frame: usize) {
        self.add_milestone(frame, MilestoneType::UserGenerated);
    }
    
    pub fn add_from_c3d(&mut self, frame: usize) {
        self.add_milestone(frame, MilestoneType::FromC3d);
    }

    pub(crate) fn remove_milestone(&mut self, frame: usize) {
        self.milestones.remove(&frame);
        self.dirty = true;
    }

    pub(crate) fn remove_all_milestones(&mut self) {
        self.milestones.clear();
        self.dirty = true;
    }

    pub(crate) fn get_milestones(&self) -> Vec<&usize> {
        let mut milestones = self.milestones.keys().collect::<Vec<_>>();
        milestones.sort();
        milestones
    }

    pub(crate) fn get_prev_milestone(&self, frame: usize) -> usize {
        let mut prev = 0;
        let mut keys = self.milestones.keys().cloned().collect::<Vec<_>>();
        keys.sort();
        keys.iter().for_each(|&k| {
            if k < frame - 1 {
                prev = k;
                return;
            }
        });
        prev
    }
    pub(crate) fn get_next_milestone(&self, frame: usize) -> usize {
        let mut next = 0;
        let mut keys = self.milestones.keys().cloned().collect::<Vec<_>>();
        keys.sort();
        keys.iter().rev().for_each(|&k| {
            if k > frame {
                next = k;
                return;
            }
        });
        next
    }
}

// Update board
pub(crate) fn update_milestone_board(milestones: &mut Milestones, width: f32, num_frames: usize, ui: &mut Ui) {
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
            .show_x(false)
            .show_y(false)
            // .show_background(false)  // Maybe we'd like to use this
            .height(15.)
            .width(width);
        
        ui.horizontal(|ui|{
            ui.label("Events:");
            new_plot.show(ui, |plot_ui| {
                plot_ui.set_plot_bounds(PlotBounds::from_min_max([0., -1.], [num_frames as f64, 1.]));
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