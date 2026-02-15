use std::collections::HashMap;

use bevy::{
    asset::RenderAssetUsages,
    camera::visibility::NoFrustumCulling,
    color::palettes::css::GREEN,
    mesh::{Indices, PrimitiveTopology, triangle_normal},
    pbr::OpaqueRendererMethod,
    prelude::*,
};
use noise::{NoiseFn, Perlin};

use crate::{CELL_VERTEX_COUNT, CELL_VERTEX_COUNT_F32, CELL_VERTEX_SPACING};

#[derive(Resource)]
pub struct MeshCache {
    pub material: MeshMaterial3d<StandardMaterial>,
}

/// Builds a mesh of size 1.0 x 1.0, with CELL_VERTEX_COUNT number of cells within in both
/// dimensions.
///
/// Therefore its corners always are:
/// - Bottomleft: [-0.5, -0.5]
/// - Topright: [0.5, 0.5]
///
/// [`heights`] must include values in a range of -1..CELL_VERTEX_COUNT+2 (inclusive) in both
/// dimensions.
fn build_mesh_data(heights: HashMap<(i32, i32), f32>) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    let cell_count = (CELL_VERTEX_COUNT.x * CELL_VERTEX_COUNT.y) as usize;
    let triangle_count = (cell_count + 8) * 6;

    let mut positions = vec![[0., 0., 0.]; triangle_count];
    let mut normals = vec![[0., 0., 0.]; triangle_count];
    let mut tex_coords = vec![[0., 0.]; triangle_count];
    let mut indices = vec![0; triangle_count];

    let get_vertex = |x: i32, z: i32| {
        let x_pos = (x as f32) * CELL_VERTEX_SPACING - 0.5;
        let z_pos = (z as f32) * CELL_VERTEX_SPACING - 0.5;
        [x_pos, heights[&(x, z)], z_pos]
    };
    let get_normal = |x: i32, z: i32| {
        triangle_normal(get_vertex(x, z), get_vertex(x, z + 1), get_vertex(x + 1, z))
    };

    for x in 0..CELL_VERTEX_COUNT.x {
        for z in 0..CELL_VERTEX_COUNT.y {
            let i_32 = (x + z * CELL_VERTEX_COUNT.x) as u32;
            let i = i_32 as usize;

            positions[i * 6] = get_vertex(x, z);
            positions[i * 6 + 1] = get_vertex(x, z + 1);
            positions[i * 6 + 2] = get_vertex(x + 1, z);
            positions[i * 6 + 3] = get_vertex(x + 1, z + 1);
            positions[i * 6 + 4] = get_vertex(x + 1, z);
            positions[i * 6 + 5] = get_vertex(x, z + 1);

            normals[i * 6] = get_normal(x, z);
            normals[i * 6 + 1] = get_normal(x, z + 1);
            normals[i * 6 + 2] = get_normal(x + 1, z);
            normals[i * 6 + 3] = get_normal(x + 1, z + 1);
            normals[i * 6 + 4] = get_normal(x + 1, z);
            normals[i * 6 + 5] = get_normal(x, z + 1);

            tex_coords[i * 6] = [0.0, 0.0];
            tex_coords[i * 6 + 1] = [0.0, 1.0];
            tex_coords[i * 6 + 2] = [1.0, 0.0];
            tex_coords[i * 6 + 3] = [1.0, 1.0];
            tex_coords[i * 6 + 4] = [1.0, 0.0];
            tex_coords[i * 6 + 5] = [0.0, 1.0];

            let slice = &[
                i_32 * 6,
                i_32 * 6 + 1,
                i_32 * 6 + 2,
                i_32 * 6 + 3,
                i_32 * 6 + 4,
                i_32 * 6 + 5,
            ];
            indices.splice(i * 6..i * 6 + 6, slice.iter().cloned());
        }
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, tex_coords);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

pub fn build_mesh_cache(
    mut commands: Commands<'_, '_>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    _asset_server: Res<AssetServer>,
) {
    let material = MeshMaterial3d(materials.add(StandardMaterial {
        base_color: GREEN.into(),
        reflectance: 0.01,
        // base_color_texture: Some(asset_server.load("textures/rocky/rocky_terrain_02_diff_2k.jpg")),
        // metallic_roughness_texture: Some(
        //     asset_server.load("textures/rocky/rocky_terrain_02_rough_2k.jpg"),
        // ),
        // normal_map_texture: Some(
        //     asset_server.load("textures/rocky/rocky_terrain_02_nor_gl_2k.jpg"),
        // ),
        // occlusion_texture: Some(asset_server.load("textures/rocky/rocky_terrain_02_ao_2k.jpg")),
        opaque_render_method: OpaqueRendererMethod::Auto,
        ..Default::default()
    }));

    commands.insert_resource(MeshCache { material });
}

pub fn rect_to_transform(rect: Rect) -> Transform {
    Transform::from_translation(Vec3::new(rect.center().x, 0.0, rect.center().y))
        .with_scale(Vec3::new(rect.width(), 1.0, rect.height()))
}

pub fn spawn_mesh(
    commands: &mut Commands,
    meshes: &mut ResMut<'_, Assets<Mesh>>,
    root_entity: &Entity,
    mesh_cache: &Res<MeshCache>,
    rect: Rect,
) -> Entity {
    let perlin = Perlin::new(1);
    let x_rng = (rect.width() / (CELL_VERTEX_COUNT_F32.x + 0.0)) as f64;
    let z_rng = (rect.height() / (CELL_VERTEX_COUNT_F32.y + 0.0)) as f64;
    let vert_scale = 0.3;

    let heights = (-1i32..CELL_VERTEX_COUNT.x + 2)
        .flat_map(|x| {
            (-1i32..CELL_VERTEX_COUNT.y + 2).map(move |z| {
                (
                    (x, z),
                    vert_scale
                        * perlin.get([
                            rect.min.x as f64 + x as f64 * x_rng,
                            0.0,
                            rect.min.y as f64 + z as f64 * z_rng,
                        ]) as f32
                        - 0.9,
                )
            })
        })
        .collect();

    let mesh_3d = Mesh3d(meshes.add(build_mesh_data(heights)));

    let entity = commands.spawn((
        mesh_3d.clone(),
        rect_to_transform(rect),
        mesh_cache.material.clone(),
        NoFrustumCulling,
    ));
    let eid = entity.id();
    commands.entity(*root_entity).add_child(eid);
    eid
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_to_transform() {
        assert_eq!(
            rect_to_transform(Rect::from_center_size(Vec2::ZERO, Vec2::ONE)),
            Transform::IDENTITY
        );
        assert_eq!(
            rect_to_transform(Rect::from_center_size(Vec2::ONE, Vec2::ONE)),
            Transform::from_translation(Vec3::new(1.0, 0.0, 1.0))
        );
    }
}
