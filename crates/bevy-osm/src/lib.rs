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
    chunk::Chunk,
    config::OSMConfig,
    material::MapMaterialHandle,
    task_pool::{handle_tasks, load_unloaded_chunks, preload_chunks},
};
use bevy::prelude::*;
use bevy_terrain::quadtree::{QuadTree, QuadTreeConfig, QuadTreeNode};

pub struct OSMPlugin;

impl Plugin for OSMPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapMaterialHandle>()
            .init_resource::<OSMConfig>()
            .add_systems(Startup, build_terrain_tile)
            .add_systems(Update, (handle_tasks, load_unloaded_chunks, preload_chunks));
    }
}

pub fn build_terrain_tile(mut commands: Commands, osm_config: Res<OSMConfig>) {
    let chunk = Chunk {
        x: 1066,
        y: 746,
        z: 11,
        elevation: Handle::default(),
        raster: Handle::default(),
    };

    let config = QuadTreeConfig {
        k: 1.1,
        min_lod: 0,
        max_lod: 5,
        size: chunk.get_size_in_meters().x,
    };
    let area_meters = chunk.get_area_in_meters(osm_config.location.get_world_center());

    let quadtree = commands
        .spawn((
            Transform::from_scale(Vec3::new(area_meters.width(), 1.0, area_meters.height())),
            QuadTree,
            config.clone(),
            Visibility::Visible,
        ))
        .id();

    let root = commands
        .spawn((
            QuadTreeNode::new(Vec2::ZERO, chunk.get_size_in_meters(), chunk.x, chunk.y),
            Transform::IDENTITY,
        ))
        .id();
    commands.entity(quadtree).add_child(root);
}
