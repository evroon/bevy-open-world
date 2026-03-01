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
pub struct QuadTree {
    pub root: QuadTreeNode,
}

#[derive(Debug, Default, Clone)]
pub struct QuadTreeNode {
    pub entity: Option<Entity>,
    pub rect: Rect,
    pub children: Vec<Box<QuadTreeNode>>,
    pub x: i32,
    pub y: i32,
    pub lod: u8,
}

pub const QUADTREE_SIZE: f32 = 40.0;

#[derive(Component)]
pub struct QuadTreeNodeComponent {
    pub x: i32,
    pub y: i32,
    pub lod: u8,
    pub rect: Rect,
}
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

pub fn get_mesh(commands: &mut Commands, root_entity: &Entity, node: &QuadTreeNode) -> Entity {
    let entity = commands.spawn((
        rect_to_transform(node.rect),
        QuadTreeNodeComponent {
            x: node.x,
            y: node.y,
            lod: node.lod,
            rect: node.rect,
        },
    ));
    let eid = entity.id();
    commands.entity(*root_entity).add_child(eid);
    eid
}

impl QuadTreeNode {
    pub fn new(origin: Vec2, size: Vec2, x: i32, y: i32) -> Self {
        Self::new_tree_segment(&origin, &size, 0, x, y)
    }

    fn new_tree_segment(origin: &Vec2, half_size: &Vec2, lod: u8, x: i32, y: i32) -> QuadTreeNode {
        Self {
            rect: Rect::from_center_size(*origin, *half_size),
            lod,
            entity: None,
            children: Vec::new(),
            x,
            y,
        }
    }

    fn subdivide(&mut self) {
        assert!(self.children.is_empty());

        // calculate size of new segment by getting a half of the parent size
        let h = self.rect.height() / 2.0;
        let w = self.rect.width() / 2.0;
        let size = Vec2::new(w, h);

        // parent origin
        let x = self.rect.center().x;
        let y = self.rect.center().y;

        // calculate origin point for each new section (+Y (and therefore +Z) is south, X is east)
        let ne_origin = Vec2::new(x + (w / 2.0), y - (h / 2.0));
        let nw_origin = Vec2::new(x - (w / 2.0), y - (h / 2.0));
        let se_origin = Vec2::new(x + (w / 2.0), y + (h / 2.0));
        let sw_origin = Vec2::new(x - (w / 2.0), y + (h / 2.0));

        // create new tree segments
        let build_child_segment = |origin: &Vec2, x: i32, y: i32| {
            let seg = Self::new_tree_segment(origin, &size, self.lod + 1, x, y);
            Box::new(seg)
        };

        self.children = [
            build_child_segment(&ne_origin, 2 * self.x + 1, 2 * self.y),
            build_child_segment(&nw_origin, 2 * self.x, 2 * self.y),
            build_child_segment(&se_origin, 2 * self.x + 1, 2 * self.y + 1),
            build_child_segment(&sw_origin, 2 * self.x, 2 * self.y + 1),
        ]
        .into();
    }

    pub fn destruct(&mut self, root_entity: &Entity, commands: &mut Commands) {
        if let Some(entity_id) = self.entity {
            commands.get_entity(entity_id).unwrap().despawn();
            commands
                .get_entity(*root_entity)
                .unwrap()
                .detach_child(entity_id);
            self.entity = None;
        }

        for child in &mut self.children {
            child.as_mut().destruct(root_entity, commands);
        }
    }

    pub fn build_around_point(
        &mut self,
        config: &QuadTreeConfig,
        root_entity: &Entity,
        commands: &mut Commands,
        ref_point: Vec3,
        nodes_query: &Query<(Entity, Option<&Children>, Option<&ChunkLoaded>)>,
    ) {
        let increase_lod = (should_subdivide(self.rect, ref_point, config.k)
            && self.lod < config.max_lod)
            || self.lod < config.min_lod;

        if increase_lod {
            if self.children.is_empty() {
                self.subdivide();
            } else {
                let all_loaded = self.children.iter().all(|c| {
                    c.entity.is_none() || nodes_query.get(c.entity.unwrap()).unwrap().2.is_some()
                });

                if all_loaded && self.entity.is_some() {
                    commands.get_entity(self.entity.unwrap()).unwrap().despawn();
                    self.entity = None;
                }
            }
            for child in &mut self.children {
                child.build_around_point(config, root_entity, commands, ref_point, nodes_query);
            }
        } else if let Some(ent) = self.entity {
            let loaded = nodes_query.get(ent).unwrap().2.is_some();
            if loaded && !self.children.is_empty() {
                for child in &mut self.children {
                    child.destruct(root_entity, commands);
                }
                self.children = Vec::new();
            }
        } else {
            self.entity = Some(get_mesh(commands, root_entity, self));
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
