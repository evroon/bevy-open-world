pub mod camera;
pub mod material;
pub mod mesh;
pub mod quadtree;
pub mod system;
pub mod water;

use bevy::{pbr::ExtendedMaterial, prelude::*};

use quadtree::{QuadTree, QuadTreeConfig, QuadTreeNode};

use crate::{mesh::build_mesh_cache, system::update_terrain_quadtree, water::Water};

pub struct WaterPlugin;

impl Plugin for WaterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<ExtendedMaterial<StandardMaterial, Water>>::default());
    }
}
pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, build_mesh_cache)
            .add_systems(Update, update_terrain_quadtree);
    }
}

pub fn build_terrain_tile(mut commands: Commands) {
    let size = 40.0;

    let config = QuadTreeConfig {
        k: 1.1,
        max_lod: 24,
        min_lod: 0,
        size,
    };
    let quadtree = QuadTree {
        root: QuadTreeNode::new(Vec2::ZERO, Vec2::splat(size), 0, 0),
    };

    commands.spawn((
        Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
        quadtree,
        config,
        Visibility::Inherited,
    ));
}
