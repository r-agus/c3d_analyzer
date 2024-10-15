mod c3d_config;

use bevy_app::{App, Plugin, Update};

pub mod prelude {
    pub use crate::c3d_config::*;
}

pub use prelude::*;

/// Plugin for configuration of C3D files
#[derive(Default)]
pub struct C3dConfigPlugin;

/// Required components for loading C3D configuration files
impl Plugin for C3dConfigPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ConfigState>();
    }
}