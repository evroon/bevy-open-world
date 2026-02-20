pub mod chunk;
pub mod config;
pub mod elevation;
pub mod location;
pub mod mesh;
pub mod osm_types;
mod theme;

extern crate osm_xml as osm;
use crate::{
    config::OSMConfig,
    material::MapMaterialHandle,
    task_pool::{handle_tasks, spawn_task},
};
use bevy::prelude::*;

mod building;
mod material;
mod task_pool;
mod tile;

pub struct OSMPlugin;

impl Plugin for OSMPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapMaterialHandle>()
            .init_resource::<OSMConfig>()
            .add_systems(Startup, spawn_task)
            .add_systems(Update, handle_tasks);
    }
}
