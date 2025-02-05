/// # Milestones
/// In this module we define the Milestones resource and its methods.
/// In a common c3d file, there are defined some events. As in a bevy context events has a special meaning, we call them milestones.
/// So milestones are events that are defined in the c3d file.

use std::{collections::HashMap, usize};

use bevy::prelude::Resource;
use bevy_egui::egui::Ui;
use egui_plot::{Line, PlotBounds};

#[derive(Resource, Default)]
pub struct Milestones {
    milestones: HashMap<usize, MilestoneType>,  // Frame, MilestoneType
}

enum MilestoneType {
    UserGenerated,
    FromC3d,
}

impl Milestones {
    pub fn default(&mut self) {
        self.milestones = HashMap::new();
    }

    fn add_milestone(&mut self, frame: usize, milestone_type: MilestoneType) {
        self.milestones.insert(frame, milestone_type);
    }

    pub(crate) fn add_user_generated(&mut self, frame: usize) {
        self.add_milestone(frame, MilestoneType::UserGenerated);
    }
    
    pub fn add_from_c3d(&mut self, frame: usize) {
        self.add_milestone(frame, MilestoneType::FromC3d);
    }

    pub(crate) fn remove_milestone(&mut self, frame: usize) {
        self.milestones.remove(&frame);
    }

    pub(crate) fn _remove_all_milestones(&mut self) {
        self.milestones.clear();
    }

    pub(crate) fn remove_user_generated_milestones(&mut self){
        self.milestones.retain(|_, v| match v {
            MilestoneType::UserGenerated => false,
            _ => true,
        });
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
pub(crate) fn update_milestone_board(milestones: &&mut Milestones, width: f32, num_frames: usize, ui: &mut Ui) {
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
}


pub(crate) fn milestones_event_orchestrator(
    mut milestones: bevy::prelude::ResMut<Milestones>,
    mut event_reader: bevy::prelude::EventReader<control_plugin::MilestoneEvent>
){
    for event in event_reader.read() {
        match event {
            control_plugin::MilestoneEvent::AddMilestoneFromC3dEvent(frame) => milestones.add_from_c3d(*frame),
            _ => {}
        }
    }
}