pub mod building;
pub mod cache;
pub mod chunk;
pub mod config;
pub mod elevation;
pub mod layer;
pub mod load_data;
pub mod location;
pub mod material;
pub mod mesh;
pub mod osm_types;
pub mod performance;
pub mod tag;
pub mod theme;
pub mod ui;
pub mod vector;

extern crate osm_xml as osm;
use crate::{
    cache::ensure_session_is_valid,
    chunk::Chunk,
    config::OSMConfig,
    load_data::{handle_vector_tasks, load_unloaded_chunks, preload_chunks},
    material::MapMaterialHandle,
    performance::{OSMPerformance, update_performance},
    ui::setup_osm_ui,
};
use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
use bevy_egui::EguiPrimaryContextPass;
use bevy_terrain::{
    mesh::build_mesh_cache,
    quadtree::{QuadTree, QuadTreeConfig, QuadTreeNode},
    system::update_terrain_quadtree,
};

pub struct OSMPlugin;

impl Plugin for OSMPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapMaterialHandle>()
            .init_resource::<OSMConfig>()
            .init_resource::<OSMPerformance>()
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(EguiPrimaryContextPass, setup_osm_ui)
            .add_systems(Startup, (build_terrain_tile, build_mesh_cache))
            .add_systems(
                Update,
                (
                    update_terrain_quadtree,
                    handle_vector_tasks.before(update_terrain_quadtree),
                    load_unloaded_chunks.before(update_terrain_quadtree),
                    preload_chunks.before(update_terrain_quadtree),
                    update_performance,
                ),
            );
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
        max_lod: 13,
        size: chunk.get_size_in_meters().x,
    };
    let area_meters = chunk.get_area_in_meters(osm_config.location.get_world_center());
    let quadtree = QuadTree {
        root: QuadTreeNode::new(Vec2::ZERO, area_meters.size(), chunk.x, chunk.y),
    };
    ensure_session_is_valid(&osm_config.raster_tile_source);

    commands.spawn((
        Transform::IDENTITY,
        quadtree.clone(),
        config.clone(),
        Visibility::Inherited,
    ));
}
