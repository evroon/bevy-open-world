use bevy::{
    prelude::*,
    render::{render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
};

use super::mesh::update_mesh;

#[derive(Resource)]
struct MeshCache {
    pub mesh: Mesh,
    pub meshes: Vec<Mesh3d>,
    pub meshes_used: Vec<bool>,
}

impl MeshCache {
    pub fn new() -> Self {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        );
        update_mesh(&mut mesh);

        MeshCache {
            mesh,
            meshes: Vec::new(),
            meshes_used: Vec::new(),
        }
    }

    pub fn get_mesh_from_cache() {}

    pub fn mark_mesh_as_unused(&mut self) {
        self.meshes[i]
    }
}

fn setup_mesh_cache(mut commands: Commands) {
    commands.insert_resource(MeshCache {});
}
