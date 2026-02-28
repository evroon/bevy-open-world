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
    chunk::{Chunk, LAT_LON_TO_METERS_CONVERSION},
    config::OSMConfig,
    material::MapMaterialHandle,
    task_pool::{handle_tasks, load_unloaded_chunks, preload_chunks},
};
use bevy::{math::ops::powf, prelude::*};
use bevy_terrain::quadtree::{MeshPool, QuadTree, QuadTreeConfig, QuadTreeNode};

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
    commands.spawn(MeshPool::new());

    let chunk = Chunk {
        x: 0,
        y: 0,
        z: 0,
        elevation: Handle::default(),
        raster: Handle::default(),
    };
    let chunk2 = Chunk {
        x: 1066,
        y: 746,
        z: 11,
        elevation: Handle::default(),
        raster: Handle::default(),
    };

    let n = powf(2.0, chunk2.z as f32);
    let x = 2.0 * ((chunk2.x as f32 + 0.5) / n - 0.5);
    let y = 2.0 * ((chunk2.y as f32 + 0.5) / n - 0.5);

    let config = QuadTreeConfig {
        k: 1.1,
        min_lod: 0,
        max_lod: 16,
        size: chunk.get_size_in_meters(),
    };
    let area_meters = chunk.get_area_in_meters(osm_config.location.get_world_center());
    let quadtree = QuadTree {
        root: QuadTreeNode::new(Vec2::ZERO, area_meters.size(), chunk.x, chunk.y),
    };

    commands.spawn((
        Transform::from_translation(Vec3::new(
            LAT_LON_TO_METERS_CONVERSION.x * 85.0 * y,
            0.0,
            -LAT_LON_TO_METERS_CONVERSION.x * 180.0 * x,
        )),
        quadtree.clone(),
        config.clone(),
        Visibility::Inherited,
    ));
}
