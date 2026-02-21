mod building;
pub mod chunk;
pub mod config;
pub mod elevation;
pub mod location;
pub mod material;
pub mod mesh;
pub mod osm_types;
pub mod task_pool;
mod theme;
mod tile;

extern crate osm_xml as osm;
use crate::{
    config::OSMConfig,
    material::MapMaterialHandle,
    task_pool::{handle_tasks, load_unloaded_chunks, preload_chunk},
};
use bevy::prelude::*;

pub struct OSMPlugin;

impl Plugin for OSMPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapMaterialHandle>()
            .init_resource::<OSMConfig>()
            .add_systems(Startup, load_chunk_monaco)
            .add_systems(Update, (handle_tasks, load_unloaded_chunks));
    }
}

pub fn load_chunk_monaco(
    commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<OSMConfig>,
) {
    preload_chunk(commands, asset_server, config.location.get_chunk());
}
