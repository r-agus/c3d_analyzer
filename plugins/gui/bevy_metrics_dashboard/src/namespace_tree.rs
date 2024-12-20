use crate::{
    dashboard_window::RequestPlot,
    registry::{MetricsRegistry, SearchResult},
};
use bevy::{
    prelude::*,
    tasks::{block_on, AsyncComputeTaskPool, Task},
};
use bevy_egui::{
    egui::{self, Ui},
    EguiContexts,
};
use std::{
    borrow::BorrowMut, sync::atomic::{AtomicU64, Ordering}, time::{Duration, Instant}
};

use control_plugin::{AppState, TraceEvent, TraceInfo};

/// A widget that shows all metrics metadata in a tree, grouped by namespace.
///
/// For example, a metric with name "foo::bar::baz" would be found by expanding
/// "foo", then "bar", then "baz".
#[derive(Component)]
pub struct NamespaceTreeWindow {
    title: String,
    id: egui::Id,
    force_refresh: bool,
    refresh_period: Duration,
    last_refresh_time: Instant,
    refresh_task: Option<Task<Vec<NamespaceNode>>>,
    roots: Vec<NamespaceNode>,
}

impl NamespaceTreeWindow {
    pub fn new(title: impl Into<String>) -> Self {
        static WINDOW_ID: AtomicU64 = AtomicU64::new(0);
        let id = WINDOW_ID.fetch_add(1, Ordering::Relaxed);
        let title = title.into();
        let id = format!("{title} {id}").into();
        Self {
            title,
            id,
            force_refresh: true,
            refresh_period: Duration::from_millis(500),
            last_refresh_time: Instant::now(),
            refresh_task: Default::default(),
            roots: Default::default(),
        }
    }

    pub fn force_refresh(&mut self) {
        self.force_refresh = true;
    }

    pub fn set_refresh_period(&mut self, period: Duration) {
        self.refresh_period = period;
    }

    pub fn restart_everything(&mut self) {
        self.roots.clear();
        self.refresh_task = None;
    }

    pub(crate) fn draw_all(
        mut trace_event: EventWriter<TraceEvent>,
        mut commands: Commands,
        registry: Res<MetricsRegistry>,
        mut app_state: ResMut<AppState>,
        mut ctxts: EguiContexts,
        mut requests: EventWriter<RequestPlot>,
        mut windows: Query<(Entity, &mut Self)>,
    ) {
        let ctxt = ctxts.ctx_mut();
        for (entity, mut window) in &mut windows {
            let mut open = true;
            egui::Window::new(&window.title)
                .id(window.id)
                .open(&mut open)
                .show(ctxt, |ui| {
                    let result = window.draw(&registry, app_state.borrow_mut(), &mut trace_event, ui);
                    if let Some(result) = result {
                        requests.send(RequestPlot {
                            key: result.key,
                            unit: result.description.and_then(|d| d.unit),
                        });
                    }
                });
            if !open {
                commands.entity(entity).despawn();
            }
        }
    }

    /// Draw the widget and accept user input.
    ///
    /// If the user selects a metric, it will be returned.
    pub fn draw(&mut self, registry: &MetricsRegistry, app_state: &mut AppState, trace_event: &mut EventWriter<TraceEvent>, ui: &mut Ui) -> Option<SearchResult> {
        if self.force_refresh || self.last_refresh_time.elapsed() > self.refresh_period {
            self.force_refresh = false;
            let task_registry = registry.clone();
            self.refresh_task = Some(AsyncComputeTaskPool::get().spawn(async move {
                let mut results = task_registry.all_metrics();
                NamespaceNode::tree_from_results(&mut results)
            }));
            self.last_refresh_time = Instant::now();
        }

        // Check if we have new search results.
        if let Some(task) = self.refresh_task.take() {
            if task.is_finished() {
                self.roots = block_on(task);
            } else {
                self.refresh_task = Some(task);
            }
        }

        let mut selected = None;
        egui::ScrollArea::new([false, true]).show(ui, |ui| {
            Self::draw_recursive(&self.roots, &mut selected, &mut Some(&mut app_state.traces), trace_event, ui);
        });
        selected
    }

    fn draw_recursive(nodes: &[NamespaceNode], selected_plot: &mut Option<SearchResult>, selected_trace: &mut Option<&mut TraceInfo>, trace_event: &mut EventWriter<TraceEvent>, ui: &mut Ui) {
        for node in nodes {
            match node {
                NamespaceNode::Namespace {
                    display_path: path_component,
                    children,
                } => {
                    ui.horizontal(|ui| {
                        let text = if selected_trace
                            .as_ref()
                            .is_some_and(|s| !s.is_trace_added(path_component.to_string())) 
                                {
                                    "Trace"
                                } else {
                                    "Remove Trace"
                                };
                        if ui.button(text).clicked() {
                            if let Some(trace) = selected_trace.as_mut() {
                                if !trace.is_trace_added(path_component.to_string()){
                                    trace.add_point(path_component.to_string());
                                    trace_event.send(TraceEvent::UpdateTraceEvent);
                                } else{
                                    trace_event.send(TraceEvent::DespawnTraceEvent(path_component.to_string()));
                                }
                            } 
                        }
                        ui.collapsing(path_component, |ui| {
                            Self::draw_recursive(children, selected_plot, &mut None, trace_event, ui);
                        });
                    });

                }
                NamespaceNode::Metric {
                    display_path,
                    result,
                } => {
                    ui.horizontal(|ui| {
                        if ui.button("Plot").clicked() {
                            *selected_plot = Some(result.clone());
                        }
                        ui.label(result.detailed_text(Some(display_path)));
                    });
                }
            }
        }
    }
}

enum NamespaceNode {
    Namespace {
        display_path: String,
        children: Vec<NamespaceNode>,
    },
    Metric {
        display_path: String,
        result: SearchResult,
    },
}

impl NamespaceNode {
    fn tree_from_results(results: &mut [SearchResult]) -> Vec<Self> {
        results.sort_unstable_by(|r1, r2| r1.key.key.name().cmp(r2.key.key.name()));
        Self::tree_from_sorted_results_recursive(results, 0)
    }

    fn tree_from_sorted_results_recursive(
        mut results: &[SearchResult],
        path_start: usize,
    ) -> Vec<Self> {
        const DELIM: &str = "::";

        // Having sorted results allows us to create a subtree from a range
        // of results.
        //
        // The results themselves are cheap to clone, so we take the easier
        // tactic of cloning results into the tree instead of moving them.
        let mut nodes = Vec::new();
        while let Some(first_result) = results.first() {
            let first_path = &first_result.key.key.name()[path_start..];
            if first_path.starts_with(':') {
                // Skip invalid path.
                results = &results[1..];
                continue;
            }

            if let Some((group_name, _)) =
                first_result.key.key.name()[path_start..].split_once(DELIM)
            {
                // Split a group off the front of the results.
                let group_end = results
                    .iter()
                    .position(|r| !r.key.key.name()[path_start..].starts_with(group_name))
                    .unwrap_or(results.len());
                let (group, rem) = results.split_at(group_end);

                // Recurse and create node from children.
                let new_path_start = path_start + group_name.len() + DELIM.len();
                let children = Self::tree_from_sorted_results_recursive(group, new_path_start);
                if let Some(node) = Self::create_parent_node(group_name, children) {
                    nodes.push(node);
                }
                results = rem;
            } else {
                // No delimiter. This result is a leaf.
                let (leaf_result, rem) = results.split_first().unwrap();
                let leaf_name = leaf_result.key.key.name();
                let is_invalid_path = leaf_name.is_empty() || leaf_name.ends_with(':');
                if !is_invalid_path {
                    // Only display last component of path.
                    let display_path = leaf_result
                        .key
                        .key
                        .name()
                        .rsplit_once(':')
                        .map(|(_, end)| end)
                        .unwrap_or(leaf_result.key.key.name());
                    nodes.push(Self::Metric {
                        display_path: display_path.into(),
                        result: leaf_result.clone(),
                    });
                }
                results = rem;
            }
        }
        nodes
    }

    fn create_parent_node(group_name: &str, children: Vec<Self>) -> Option<Self> {
        match children.len() {
            0 => None,
            1 => {
                let collapsed = match children.into_iter().next().unwrap() {
                    Self::Namespace {
                        display_path: path_component,
                        children,
                    } => Self::Namespace {
                        display_path: format!("{group_name}::{path_component}"),
                        children,
                    },
                    Self::Metric {
                        display_path,
                        result,
                    } => Self::Metric {
                        display_path: format!("{group_name}::{display_path}"),
                        result,
                    },
                };
                Some(collapsed)
            }
            _ => Some(Self::Namespace {
                display_path: group_name.into(),
                children,
            }),
        }
    }
}
