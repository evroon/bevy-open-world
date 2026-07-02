pub mod building;
pub mod cache;
pub mod chunk;
pub mod config;
pub mod elevation;
pub mod load_data;
pub mod location;
pub mod material;
pub mod mesh;
pub mod osm_types;
pub mod performance;
pub mod schema;
pub mod tag;
pub mod theme;
pub mod ui;
pub mod vector;

use crate::{
    cache::ensure_session_is_valid,
    chunk::{get_chunk_for_coord, get_root_chunk_for_location},
    config::OSMConfig,
    load_data::{handle_vector_tasks, load_unloaded_chunks, preload_chunks},
    material::{MapMaterialHandle, MapMeshHandle},
    performance::{OSMPerformance, update_performance},
    ui::setup_osm_ui,
};
use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
use bevy_egui::EguiPrimaryContextPass;
use bevy_terrain::{
    mesh::build_mesh_cache,
    quadtree::{QuadTree, QuadTreeConfig},
    system::update_terrain_quadtree,
};

pub struct OSMPlugin;

impl Plugin for OSMPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapMaterialHandle>()
            .init_resource::<MapMeshHandle>()
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
    let origin = osm_config.location.get_world_center();

    let config = QuadTreeConfig {
        k: 1.1,
        min_lod: 0,
        max_lod: 13,
        size: get_chunk_for_coord(origin.x as f64, origin.y as f64, 9)
            .get_size_in_meters()
            .x,
    };

    ensure_session_is_valid(&osm_config.raster_tile_source);

    commands.spawn((
        Transform::IDENTITY,
        QuadTree {
            root: get_root_chunk_for_location(&osm_config.location),
        },
        config.clone(),
        Visibility::Inherited,
    ));
}
