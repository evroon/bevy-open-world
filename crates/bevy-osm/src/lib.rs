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
pub mod ui;

extern crate osm_xml as osm;
use crate::{
    chunk::Chunk,
    config::OSMConfig,
    material::MapMaterialHandle,
    task_pool::{handle_tasks, load_unloaded_chunks, preload_chunks},
    ui::setup_osm_ui,
};
use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
use bevy_terrain::quadtree::{QuadTree, QuadTreeConfig, QuadTreeNode};

pub struct OSMPlugin;

impl Plugin for OSMPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapMaterialHandle>()
            .init_resource::<OSMConfig>()
            .add_systems(EguiPrimaryContextPass, setup_osm_ui)
            .add_systems(Startup, build_terrain_tile)
            .add_systems(Update, (handle_tasks, load_unloaded_chunks, preload_chunks));
    }
}

pub fn build_terrain_tile(mut commands: Commands, osm_config: Res<OSMConfig>) {
    let chunk = Chunk {
        x: 266,
        y: 186,
        z: 9,
        elevation: Handle::default(),
        raster: Handle::default(),
    };

    let config = QuadTreeConfig {
        k: 1.1,
        min_lod: 0,
        max_lod: 7,
        size: chunk.get_size_in_meters().x,
    };
    let area_meters = chunk.get_area_in_meters(osm_config.location.get_world_center());
    let quadtree = QuadTree {
        root: QuadTreeNode::new(Vec2::ZERO, area_meters.size(), chunk.x, chunk.y),
    };

    commands.spawn((
        Transform::IDENTITY,
        quadtree.clone(),
        config.clone(),
        Visibility::Inherited,
    ));
}
