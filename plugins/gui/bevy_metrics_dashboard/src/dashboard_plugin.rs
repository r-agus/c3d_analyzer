use crate::{
    dashboard_window::{CachedPlotConfigs, RequestPlot},
    namespace_tree::NamespaceTreeWindow,
    ClearBucketsSystem, DashboardWindow,
};
use bevy::prelude::*;

/// Updates and renders all [`DashboardWindow`] and [`NamespaceTreeWindow`]
/// entities.
pub struct DashboardPlugin;

impl Plugin for DashboardPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<RequestPlot>()
            .init_resource::<CachedPlotConfigs>()
            .add_systems(
                Update,
                NamespaceTreeWindow::draw_all, // Removed DashboardWindow::draw_all 
            )
            // Enforce strict ordering:s
            // metrics producers (before Last) --> metrics consumers --> bucket clearing
            .add_systems(Last, DashboardWindow::update_all_graphs_play_pause.before(ClearBucketsSystem));
    }
}
