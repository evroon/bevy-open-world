use bevy::prelude::*;

use crate::mesh::rect_to_transform;

#[derive(Component, Debug, Default, Clone)]
pub struct QuadTreeConfig {
    /// determines tesselation sensitivity, should be larger than 1.0
    pub k: f32,
    pub max_lod: u8,
    pub min_lod: u8,
    /// We assume a world where:
    ///   - 1 unit = 1 meter
    ///   - earth with a circumference of 40_000_000 meter
    ///   - cell size of 256 units/meters
    ///   - 18 max subdivision levels
    ///   - A cube => sphere deformation, where horizontal circumferece is covered by 4 planes
    ///
    /// Then: 2^18 * 4 / 40_000_000 * 256 == 6.7
    /// So, we'll have 6.7 vertices per meter, or 1 vertex per 15 centimeter at max LOD (LOD 20).
    pub size: f32,
}

#[derive(Component, Debug, Default, Clone)]
pub struct QuadTree;

#[derive(Component, Debug, Default, Clone)]
pub struct QuadTreeNode {
    pub rect: Rect,
    pub x: i32,
    pub y: i32,
    pub lod: u8,
}

pub const QUADTREE_SIZE: f32 = 40.0;

#[derive(Component)]
pub struct IncreaseLOD;

#[derive(Component)]
pub struct DecreaseLOD;
#[derive(Component)]
pub struct ChunkLoaded;

/// subdivide based on non-euclidian max(dx, dy, dz) distance from camera
///
/// https://proland.inrialpes.fr/doc/proland-4.0/core/html/index.html
fn should_subdivide(object_rect: Rect, camera_position: Vec3, k: f32) -> bool {
    let d_bottom_left = (camera_position.xz() - object_rect.min).abs();
    let d_top_right = (camera_position.xz() - object_rect.size() - object_rect.min).abs();
    let d_horiz = f32::max(
        f32::min(d_bottom_left.x, d_top_right.x),
        f32::min(d_bottom_left.y, d_top_right.y),
    );
    let d = f32::max(d_horiz, camera_position.y);

    d < k * object_rect.width()
}

// pub fn get_mesh(commands: &mut Commands, root_entity: &Entity, node: QuadTreeNode) -> Entity {
//     let entity = commands.spawn((rect_to_transform(node.rect), node));
//     let eid = entity.id();
//     commands.entity(*root_entity).add_child(eid);
//     eid
// }

impl QuadTreeNode {
    pub fn new(origin: Vec2, size: Vec2, x: i32, y: i32) -> Self {
        QuadTreeNode {
            rect: Rect::from_center_size(origin, size),
            lod: 0,
            x,
            y,
        }
    }

    fn subdivide(&self, commands: &mut Commands, entity: Entity) {
        // calculate size of new segment by getting a half of the parent size
        let h = self.rect.height() / 2.0;
        let w = self.rect.width() / 2.0;
        let size_world = Vec2::new(w, h);

        // parent origin
        let x = self.rect.center().x;
        let y = self.rect.center().y;

        for (origin_world, origin_local, x, y) in [
            (
                Vec2::new(x - (w / 2.0), y + (h / 2.0)),
                Vec2::new(-0.25, 0.25),
                2 * self.x + 1,
                2 * self.y,
            ),
            (
                Vec2::new(x + (w / 2.0), y + (h / 2.0)),
                Vec2::new(0.25, 0.25),
                2 * self.x,
                2 * self.y,
            ),
            (
                Vec2::new(x - (w / 2.0), y - (h / 2.0)),
                Vec2::new(-0.25, -0.25),
                2 * self.x + 1,
                2 * self.y + 1,
            ),
            (
                Vec2::new(x + (w / 2.0), y - (h / 2.0)),
                Vec2::new(0.25, -0.25),
                2 * self.x,
                2 * self.y + 1,
            ),
        ] {
            let new = commands
                .spawn((
                    rect_to_transform(Rect::from_center_size(origin_local, Vec2::splat(0.5))),
                    QuadTreeNode {
                        rect: Rect::from_center_size(origin_world, size_world),
                        lod: self.lod + 1,
                        x,
                        y,
                    },
                ))
                .id();
            commands.get_entity(entity).unwrap().add_child(new);
        }
    }

    #[expect(clippy::type_complexity)]
    pub fn build_around_point(
        &self,
        config: &QuadTreeConfig,
        entity: Entity,
        commands: &mut Commands,
        nodes: &Query<(
            Entity,
            &mut QuadTreeNode,
            Option<&Children>,
            Option<&ChunkLoaded>,
            Option<&DecreaseLOD>,
            Option<&IncreaseLOD>,
        )>,
        ref_point: Vec3,
    ) {
        let increase_lod = (should_subdivide(self.rect, ref_point, config.k)
            && self.lod < config.max_lod)
            || self.lod < config.min_lod;

        let mut ent_cmd = commands.get_entity(entity).unwrap();

        let children_default = Children::default();
        let child_entities = nodes.get(entity).unwrap().2.unwrap_or(&children_default);

        if increase_lod {
            ent_cmd.insert_if_new(IncreaseLOD);

            if child_entities.is_empty() {
                self.subdivide(commands, entity);
            } else {
                let all_loaded = child_entities
                    .iter()
                    .map(|c| nodes.get(c).ok()?.3)
                    .all(|x| x.is_some());

                if all_loaded {
                    // ent_cmd.despawn();
                } else {
                    for ce in child_entities {
                        if let Ok(cc) = nodes.get(*ce) {
                            cc.1.build_around_point(config, *ce, commands, nodes, ref_point);
                        } else {
                            // println!("test");
                        }
                    }
                }
            }
        } else if child_entities.iter().count() > 0 {
            let (_, _, _, loaded, _, _) = nodes.get(entity).unwrap();
            if loaded.is_some() {
                // get_mesh(commands, &entity, self);
            } else {
                ent_cmd.insert_if_new(DecreaseLOD);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_lod() {
        assert!(!should_subdivide(
            Rect::from_center_size(Vec2::ZERO, Vec2::ONE,),
            Vec3::new(0.0, 0.0, 1.6),
            1.1
        ));
        assert!(should_subdivide(
            Rect::from_center_size(Vec2::ZERO, Vec2::ONE,),
            Vec3::new(0.0, 0.0, 1.59),
            1.1
        ));
    }
}
