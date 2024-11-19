use bevy::{ecs::entity::Entities, prelude::*};

#[cfg(not(target_arch = "wasm32"))]
use metrics::{describe_gauge, describe_histogram, gauge, histogram, Unit}; // Not available in web

/// Provides core metrics like frame time, entity count, etc.
pub struct CoreMetricsPlugin;

impl Plugin for CoreMetricsPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(Startup, describe_core_metrics)
            .add_systems(Update, update_core_metrics);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn describe_core_metrics() {
    describe_gauge!("frame_time", Unit::Milliseconds, "Frame time delta");
    describe_histogram!("frame_time", Unit::Milliseconds, "Frame time delta");
    describe_gauge!(
        "frames_per_second",
        Unit::CountPerSecond,
        "Frames per second"
    );
    describe_gauge!(
        "entities",
        Unit::Count,
        "The number of entities in the world"
    );
}

#[cfg(not(target_arch = "wasm32"))]
fn update_core_metrics(entities: &Entities, time: Res<Time>) {
    gauge!("entities").set(entities.len() as f64);

    let sec = time.delta_seconds_f64();
    let ms = 1000.0 * sec;
    let fps = 1.0 / sec;
    histogram!("frame_time").record(ms);
    gauge!("frame_time").set(ms);
    gauge!("frames_per_second").set(fps);
}
