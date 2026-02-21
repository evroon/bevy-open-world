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
    config::OSMConfig, elevation::spawn_elevation_meshes, material::MapMaterialHandle,
    task_pool::handle_tasks,
};
use bevy::prelude::*;

pub struct OSMPlugin;

impl Plugin for OSMPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapMaterialHandle>()
            .init_resource::<OSMConfig>()
            .add_systems(Update, (handle_tasks, spawn_elevation_meshes));
    }
}
