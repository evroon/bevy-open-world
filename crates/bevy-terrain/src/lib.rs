pub mod material;
pub mod mesh;
pub mod quadtree;
pub mod system;

use bevy::{pbr::ExtendedMaterial, prelude::*};

use crate::{material::TerrainMaterial, mesh::build_mesh_cache, system::update_quadtree};

pub const CELL_COUNT: bevy::prelude::UVec2 = UVec2::splat(4);
pub const CELL_COUNT_F32: bevy::prelude::Vec2 = Vec2::new(CELL_COUNT.x as f32, CELL_COUNT.y as f32);
pub const _CELL_COUNT_I32: bevy::prelude::IVec2 =
    IVec2::new(CELL_COUNT.x as i32, CELL_COUNT.y as i32);

pub const CELL_SIZE: f32 = 1.0 / CELL_COUNT_F32.x;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, TerrainMaterial>,
        >::default())
            .add_systems(Startup, build_mesh_cache)
            .add_systems(Update, update_quadtree);
    }
}
