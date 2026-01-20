use bevy::{
    asset::RenderAssetUsages,
    camera::visibility::NoFrustumCulling,
    color::palettes::css::PURPLE,
    mesh::{Indices, PrimitiveTopology},
    pbr::{ExtendedMaterial, OpaqueRendererMethod},
    prelude::*,
};

use crate::{CELL_VERTEX_COUNT, CELL_VERTEX_SPACING, material::PlanetMaterial};

type MeshDataResult = (usize, Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u32>);

#[derive(Resource)]
pub struct MeshCache {
    pub mesh_3d: Mesh3d,
    pub material: MeshMaterial3d<ExtendedMaterial<StandardMaterial, PlanetMaterial>>,
}

/// Builds a mesh of size 1.0 x 1.0, with CELL_VERTEX_COUNT number of cells within in both
/// dimensions.
///
/// Therefore its corners always are:
/// - Bottomleft: [-0.5, -0.5]
/// - Topright: [0.5, 0.5]
fn build_mesh_data() -> MeshDataResult {
    let cell_count = (CELL_VERTEX_COUNT.x * CELL_VERTEX_COUNT.y) as usize;
    let triangle_count = (cell_count + 8) * 6;

    let mut positions = vec![[0., 0., 0.]; triangle_count];
    let mut tex_coords = vec![[0., 0.]; triangle_count];
    let mut indices = vec![0; triangle_count];

    for x in 0..CELL_VERTEX_COUNT.x {
        for y in 0..CELL_VERTEX_COUNT.y {
            let x_pos = (x as f32) * CELL_VERTEX_SPACING - 0.5;
            let z_pos = (y as f32) * CELL_VERTEX_SPACING - 0.5;

            let i_32 = x + y * CELL_VERTEX_COUNT.x;
            let i: usize = i_32 as usize;

            positions[i * 6] = [x_pos, 0.0, z_pos];
            positions[i * 6 + 1] = [x_pos, 0.0, z_pos + CELL_VERTEX_SPACING];
            positions[i * 6 + 2] = [x_pos + CELL_VERTEX_SPACING, 0.0, z_pos];
            positions[i * 6 + 3] = [
                x_pos + CELL_VERTEX_SPACING,
                0.0,
                z_pos + CELL_VERTEX_SPACING,
            ];
            positions[i * 6 + 4] = [x_pos + CELL_VERTEX_SPACING, 0.0, z_pos];
            positions[i * 6 + 5] = [x_pos, 0.0, z_pos + CELL_VERTEX_SPACING];

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
            let i_idx_usize = i * 6;
            indices.splice(i_idx_usize..i_idx_usize + 6, slice.iter().cloned());
        }
    }

    (triangle_count, positions, tex_coords, indices)
}

pub fn build_mesh_cache(
    mut commands: Commands<'_, '_>,
    mut meshes: ResMut<'_, Assets<Mesh>>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, PlanetMaterial>>>,
) {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    let (_, positions, tex_coords, indices) = build_mesh_data();

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, tex_coords);
    mesh.insert_indices(Indices::U32(indices));

    let mesh_3d = Mesh3d(meshes.add(mesh));
    let material = MeshMaterial3d(materials.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color: PURPLE.into(),
            opaque_render_method: OpaqueRendererMethod::Auto,
            ..default()
        },
        extension: PlanetMaterial { planet_radius: 20. },
    }));

    commands.insert_resource(MeshCache { mesh_3d, material });
}

pub fn rect_to_transform(rect: Rect) -> Transform {
    Transform::from_translation(Vec3::new(rect.center().x, 0.0, rect.center().y))
        .with_scale(Vec3::new(rect.width(), rect.width(), rect.height()))
}

pub fn spawn_mesh(
    commands: &mut Commands,
    root_entity: &Entity,
    mesh_cache: &Res<MeshCache>,
    rect: Rect,
) -> Entity {
    let entity = commands.spawn((
        mesh_cache.mesh_3d.clone(),
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
