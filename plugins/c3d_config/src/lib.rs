mod c3d_config;

use bevy::{asset::{io::Reader, Asset, AssetApp, AssetLoader, AssetServer, Assets, AsyncReadExt, Handle, LoadContext}, prelude::{Commands, Local, Res, Resource}, reflect::TypePath};
use bevy_app::{App, Plugin, Update};

pub mod prelude {
    pub use crate::c3d_config::*;
}

pub use prelude::*;

#[derive(Resource, Default, Debug)]
pub struct ConfigState {
    pub path: String,
    pub handle: Handle<ConfigC3dAsset>,
    pub loaded: bool,
}

/// Plugin for configuration of C3D files
#[derive(Default)]
pub struct C3dConfigPlugin;

/// Required components for loading C3D configuration files
impl Plugin for C3dConfigPlugin {
    fn build(&self, app: &mut App) {
        // app.init_resource::<ConfigState>();
        app.init_resource::<ConfigState>()
            .register_asset_loader(ConfigAssetLoader)
            .init_asset::<ConfigC3dAsset>()
            .add_systems(Update, load_config_system);
    }
}

/// Asset that represents a text file
#[derive(Asset, TypePath)]
pub struct ConfigC3dAsset {
    /// Literal text content of the file. Left for backwards compatibility.
    pub config_str: String,
    /// Configuration of the C3D file
    pub config: ConfigFile,
}

/// Asset loader for text files
#[derive(Default)]
pub struct ConfigAssetLoader;

// Implement the AssetLoader trait
impl AssetLoader for ConfigAssetLoader {
    type Asset = ConfigC3dAsset;
    type Settings = ();
    type Error = std::io::Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            let _res = reader.read_to_end(&mut bytes).await;
            let content = String::from_utf8_lossy(&bytes).to_string();
            let config_file = parse_config(&content, false).unwrap();
            Ok(ConfigC3dAsset { 
                config_str: content, 
                config: config_file,
            })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["toml"]
    }
}

#[derive(Resource)]
pub struct TextAssetHandle(Handle<ConfigC3dAsset>);

fn _setup(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
) {
    // Load the text file
    let handle: Handle<ConfigC3dAsset> = asset_server.load("example.txt");
    commands.insert_resource(TextAssetHandle(handle));
}

pub fn text_loaded(
    text_assets: Res<Assets<ConfigC3dAsset>>,
    text_handle: Res<TextAssetHandle>,
) {
    // Check if the asset is loaded
    if let Some(_text_asset) = text_assets.get(&text_handle.0) {
        return;
    }
    println!("Text asset not loaded yet");
}

fn load_config_system(
    asset_server: Res<AssetServer>,
    text_assets: Res<Assets<ConfigC3dAsset>>,
    mut loaded: Local<bool>,
    config_state: Res<ConfigState>,
) {
    if !*loaded {
        let handle = asset_server.load::<ConfigC3dAsset>(config_state.path.clone());
        if let Some(config) = text_assets.get(&handle) {
            println!("Contenido del archivo TOML: {}", config.config_str);
        }
        *loaded = true;
    }
}
