pub mod camera;
pub mod material;
pub mod mesh;
pub mod quadtree;
pub mod system;
pub mod water;

use bevy::{pbr::ExtendedMaterial, prelude::*};

use quadtree::{MeshPool, QuadTree, QuadTreeConfig, QuadTreeNode};

use crate::{mesh::build_mesh_cache, water::Water};

pub const CELL_VERTEX_COUNT: IVec2 = IVec2::splat(8);
pub const CELL_VERTEX_COUNT_F32: Vec2 =
    Vec2::new(CELL_VERTEX_COUNT.x as f32, CELL_VERTEX_COUNT.y as f32);
pub const CELL_VERTEX_SPACING: f32 = 1.0 / CELL_VERTEX_COUNT_F32.x;

pub struct WaterPlugin;

impl Plugin for WaterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<ExtendedMaterial<StandardMaterial, Water>>::default());
    }
}
pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (build_mesh_cache, build_terrain_tile));
    }
}

pub fn build_terrain_tile(mut commands: Commands) {
    let size = 40.0;
    commands.spawn(MeshPool::new());

    let config = QuadTreeConfig {
        k: 1.1,
        max_lod: 24,
        min_lod: 0,
        size,
    };
    let quadtree = QuadTree {
        root: QuadTreeNode::new(Vec2::ZERO, Vec2::splat(size)),
    };

    commands.spawn((
        Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
        quadtree.clone(),
        config.clone(),
        Visibility::Inherited,
    ));
}
