pub mod material;
pub mod mesh;
pub mod quadtree;
pub mod system;
pub mod water;

use bevy::prelude::*;

use quadtree::{MeshPool, QuadTree, QuadTreeConfig, QuadTreeNode};

pub const CELL_VERTEX_COUNT: UVec2 = UVec2::splat(8);
pub const CELL_VERTEX_COUNT_F32: Vec2 =
    Vec2::new(CELL_VERTEX_COUNT.x as f32, CELL_VERTEX_COUNT.y as f32);
pub const CELL_VERTEX_SPACING: f32 = 1.0 / CELL_VERTEX_COUNT_F32.x;

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
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        quadtree.clone(),
        config.clone(),
        Visibility::Inherited,
    ));
}
