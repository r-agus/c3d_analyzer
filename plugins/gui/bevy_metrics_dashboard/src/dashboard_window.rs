use crate::{
    namespace_tree::NamespaceTreeWindow,
    plots::{window_size_slider, MetricPlot, MetricPlotConfig},
    registry::{MetricKey, MetricsRegistry},
    search_bar::SearchBar,
};
use bevy::{prelude::*, utils::HashMap};
use bevy_c3d_mod::{C3dAsset, C3dState};
use bevy_egui::{
    egui::{self, Ui},
    EguiContexts,
};
use metrics::Unit;

use control_plugin::{get_marker_position_on_all_frames, AppState, Marker};

#[derive(Clone, Event)]
pub struct RequestPlot {
    pub key: MetricKey,
    pub unit: Option<Unit>,
}

/// Cache of configs for plots that have been opened and removed.
#[derive(Default, Deref, DerefMut, Resource)]
pub struct CachedPlotConfigs(HashMap<MetricKey, MetricPlotConfig>);

/// An `egui` window that can search for metrics and plot them.
#[derive(Component)]
pub struct DashboardWindow {
    pub title: String,
    pub search_bar: SearchBar,
    pub plots: Vec<MetricPlot>,
    pub config: DashboardConfig,
}

#[derive(Default)]
pub struct DashboardConfig {
    pub global_window_size: Option<usize>,
    pub real_time: bool,
}

impl DashboardWindow {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            search_bar: default(),
            plots: default(),
            config: DashboardConfig{
                global_window_size: default(),
                real_time: false,
            },
        }
    }

    pub fn update_all_graphs_play_pause(
        mut windows: Query<&mut Self>,
        state: Res<AppState>,
    ) {
        for mut window in &mut windows {     
            window.update(state.frame);
        }
    }

    pub fn update_all_graphs(
        mut windows: Query<&mut Self>,
        state: Res<AppState>,
    ) {
        for mut window in &mut windows {
            window.update(state.frame);
        }
    }

    pub(crate) fn update(&mut self, frame: usize) {
        for plot in &mut self.plots {
            plot.update(frame);
        }
    }

    pub fn draw_all(
        mut commands: Commands,
        registry: Res<MetricsRegistry>,
        mut cached_configs: ResMut<CachedPlotConfigs>,
        mut ctxts: EguiContexts,
        mut requests: EventReader<RequestPlot>,
        mut windows: Query<(Entity, &mut Self)>, 
        c3d_state: Res<C3dState>,
        c3d_assets: Res<Assets<C3dAsset>>,
        query: Query<(&Marker, &Transform)>,
    ) {
        let requests: Vec<_> = requests.read().cloned().collect();
        let ctxt = ctxts.ctx_mut();

        for (_entity, mut window) in &mut windows {
            for RequestPlot { key, unit } in requests.iter().cloned() {
                let label = key.key.name();
                let xyz = match label.chars().last() {
                    Some('x') => 0,
                    Some('y') => 1,
                    _ => 2,
                };
                let label = label.trim_end_matches("::x")
                                       .trim_end_matches("::y")
                                       .trim_end_matches("::z");

                let values = get_marker_position_on_all_frames(label, &c3d_state, &c3d_assets, &query)
                    .unwrap()
                    .iter()
                    .map(|vec3| match xyz {
                        0 => vec3.x as f64,
                        1 => vec3.y as f64,
                        _ => vec3.z as f64,
                    })
                    .collect::<Vec<f64>>();
                    
                window.add_plot(&registry, &cached_configs, key, unit, values.clone());         // This is called when clicking on a metric in the namespace tree. key contains the label of the Marker and "::x" or "::y" or "::z"
            }
            egui::SidePanel::left(window.title.clone())             
                .show(ctxt, |ui| {
                    ui.horizontal(|ui| {
                        window.add_search_results(&registry, &cached_configs, ui, &c3d_state, &c3d_assets, &query); 
                        if ui.button("Browse").clicked() {
                            commands.spawn(NamespaceTreeWindow::new("Namespace Viewer"));
                        }
                    });
                    ui.collapsing("Global Settings", |ui| {
                        window.configure_ui(ui);
                    });
                    ui.separator();
                    window.draw_plots(&mut cached_configs, ui);
                });
            // if !open {
            //     commands.entity(entity).despawn();
            // }
        }
    }

    pub(crate) fn add_search_results(
        &mut self,
        registry: &MetricsRegistry,
        cached_configs: &CachedPlotConfigs,
        ui: &mut Ui,
        c3d_state: &Res<C3dState>,
        c3d_assets: &Res<Assets<C3dAsset>>,
        query: &Query<(&Marker, &Transform)>,
    ) {
        let Some(selected) = self.search_bar.draw(registry, ui) else {
            return;
        };
        let label = selected.key.key.name();
        let xyz = match label.chars().last() {
            Some('x') => 0,
            Some('y') => 1,
            _ => 2,
        };
        let label = label.trim_end_matches("::x")
                               .trim_end_matches("::y")
                               .trim_end_matches("::z");

        let values = get_marker_position_on_all_frames(label, &c3d_state, &c3d_assets, &query)
            .unwrap()
            .iter()
            .map(|vec3| match xyz {
                0 => vec3.x as f64,
                1 => vec3.y as f64,
                _ => vec3.z as f64,
            })
            .collect::<Vec<f64>>();
        self.add_plot(
            registry,
            cached_configs,
            selected.key,
            selected.description.and_then(|d| d.unit),
            values,
        );
    }

    fn add_plot(
        &mut self,
        registry: &MetricsRegistry,
        cached_configs: &CachedPlotConfigs,
        key: MetricKey,
        unit: Option<Unit>,
        values: Vec<f64>,
    ) {
        // If we already have this metric, give it a unique name.
        let n_duplicates = self.plots.iter().filter(|p| p.key() == &key).count();

        let plot_config = cached_configs
            .get(&key)
            .cloned()
            .unwrap_or_else(|| MetricPlotConfig::default_for_kind(key.kind));
        self.plots.push(MetricPlot::new(
            registry,
            key.title(None, n_duplicates),
            key,
            unit,
            plot_config,
            self.config.real_time,
            values,
        ));
    }

    pub(crate) fn configure_ui(&mut self, ui: &mut Ui) {
        ui.checkbox(&mut self.config.real_time, "Real Time");
        self.plots.iter_mut().for_each(|p| p.real_time = self.config.real_time);

        let mut lock_window_size = self.config.global_window_size.is_some();
        ui.checkbox(&mut lock_window_size, "Link X Axes");
        if lock_window_size {
            let window_size = self.config.global_window_size.get_or_insert(500);
            ui.add(window_size_slider(window_size));
        } else {
            self.config.global_window_size = None;
        }
    }

    pub(crate) fn draw_plots(&mut self, cached_configs: &mut CachedPlotConfigs, ui: &mut Ui) {
        let mut remove_plots = Vec::new();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (i, plot) in self.plots.iter_mut().enumerate().rev() {
                // TODO: avoid string copy here?
                ui.collapsing(plot.name().to_owned(), |ui| {
                    if ui.button("Remove").clicked() {
                        remove_plots.push(i);
                    }

                    plot.draw(&self.config, ui);
                });
            }
        });

        for i in remove_plots {
            let plot = self.plots.remove(i);
            cached_configs.insert(plot.key().clone(), plot.clone_config());
        }
    }
}
