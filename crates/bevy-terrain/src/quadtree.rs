use std::collections::VecDeque;

use bevy::prelude::*;

use super::mesh::{MeshCache, rect_to_transform, spawn_mesh};

#[derive(Component, Debug, Default)]
pub struct MeshPool(pub VecDeque<Entity>);

#[derive(Component, Debug, Default, Clone)]
pub struct QuadTreeConfig {
    /// determines tesselation sensitivity, should be larger than 1.0
    pub k: f32,
    pub max_lod: u32,
    pub min_lod: u32,
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
    pub lod: u32,
    pub rect: Rect,
    pub north_east: Option<Box<QuadTreeNode>>,
    pub north_west: Option<Box<QuadTreeNode>>,
    pub south_east: Option<Box<QuadTreeNode>>,
    pub south_west: Option<Box<QuadTreeNode>>,
}

pub const QUADTREE_SIZE: f32 = 40.0;

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

impl MeshPool {
    pub fn new() -> Self {
        Self(VecDeque::new())
    }

    pub fn get_mesh(
        &mut self,
        commands: &mut Commands,
        root_entity: &Entity,
        mesh_cache: &Res<MeshCache>,
        rect: Rect,
    ) -> Entity {
        if let Some(el) = self.0.pop_front() {
            let mut root_entity = commands.get_entity(*root_entity).unwrap();
            root_entity.add_child(el);

            if let Ok(mut entity) = commands.get_entity(el) {
                entity.remove::<Visibility>();
                entity.insert(Visibility::Visible);

                entity.remove::<Transform>();
                entity.insert(rect_to_transform(rect));
            }
            return el;
        }
        spawn_mesh(commands, root_entity, mesh_cache, rect)
    }

    fn despawn_mesh(&mut self, commands: &mut Commands, entity_id: Entity) {
        if let Ok(mut entity) = commands.get_entity(entity_id) {
            entity.insert(Visibility::Hidden);
            self.0.push_back(entity_id);
        } else {
            panic!("could not despawn mesh");
        }
    }
}

impl QuadTreeNode {
    pub fn new(origin: Vec2, size: Vec2) -> Self {
        Self::new_tree_segment(&origin, &size, 0)
    }

    fn new_tree_segment(origin: &Vec2, half_size: &Vec2, lod: u32) -> QuadTreeNode {
        Self {
            rect: Rect::from_center_size(*origin, *half_size),
            lod,
            entity: None,
            north_east: None,
            north_west: None,
            south_east: None,
            south_west: None,
        }
    }

    fn subdivide(&mut self) {
        assert!(self.north_east.is_none());

        // calculate size of new segment by getting a half of the parent size
        let h = self.rect.height() / 2.0;
        let w = self.rect.width() / 2.0;
        let size = Vec2::new(w, h);

        // parent origin
        let x = self.rect.center().x;
        let y = self.rect.center().y;

        // calculate origin point for each new section
        let ne_origin = Vec2::new(x - (w / 2.0), y + (h / 2.0));
        let nw_origin = Vec2::new(x + (w / 2.0), y + (h / 2.0));
        let se_origin = Vec2::new(x - (w / 2.0), y - (h / 2.0));
        let sw_origin = Vec2::new(x + (w / 2.0), y - (h / 2.0));

        // create new tree segments
        let build_child_segment = |origin: &Vec2| {
            let seg = Self::new_tree_segment(origin, &size, self.lod + 1);
            Some(Box::new(seg))
        };

        self.north_east = build_child_segment(&ne_origin);
        self.north_west = build_child_segment(&nw_origin);
        self.south_east = build_child_segment(&se_origin);
        self.south_west = build_child_segment(&sw_origin);
    }

    pub fn destruct(
        &mut self,
        mesh_pool: &mut MeshPool,
        root_entity: &Entity,
        commands: &mut Commands,
    ) {
        if let Some(entity_id) = self.entity {
            mesh_pool.despawn_mesh(commands, entity_id);
            commands
                .get_entity(*root_entity)
                .unwrap()
                .detach_child(entity_id);
            self.entity = None;
        }

        if let Some(north_east) = &mut self.north_east {
            north_east.destruct(mesh_pool, root_entity, commands);
            self.north_west
                .as_mut()
                .unwrap()
                .destruct(mesh_pool, root_entity, commands);
            self.south_east
                .as_mut()
                .unwrap()
                .destruct(mesh_pool, root_entity, commands);
            self.south_west
                .as_mut()
                .unwrap()
                .destruct(mesh_pool, root_entity, commands);
        }
    }

    pub fn build_around_point(
        &mut self,
        config: &QuadTreeConfig,
        root_entity: &Entity,
        mesh_pool: &mut MeshPool,
        commands: &mut Commands,
        mesh_cache: &Res<MeshCache>,
        ref_point: Vec3,
    ) {
        let _should_subdivide = should_subdivide(self.rect, ref_point, config.k);
        let increase_lod =
            (_should_subdivide && self.lod < config.max_lod) || self.lod < config.min_lod;

        if increase_lod {
            if let Some(prev_en) = self.entity {
                mesh_pool.despawn_mesh(commands, prev_en);
                self.entity = None;
            }

            if self.north_east.is_none() {
                self.subdivide();
            }

            self.north_east.as_mut().unwrap().build_around_point(
                config,
                root_entity,
                mesh_pool,
                commands,
                mesh_cache,
                ref_point,
            );
            self.north_west.as_mut().unwrap().build_around_point(
                config,
                root_entity,
                mesh_pool,
                commands,
                mesh_cache,
                ref_point,
            );
            self.south_east.as_mut().unwrap().build_around_point(
                config,
                root_entity,
                mesh_pool,
                commands,
                mesh_cache,
                ref_point,
            );
            self.south_west.as_mut().unwrap().build_around_point(
                config,
                root_entity,
                mesh_pool,
                commands,
                mesh_cache,
                ref_point,
            );
        } else {
            if let Some(north_east) = &mut self.north_east {
                north_east.destruct(mesh_pool, root_entity, commands);
                self.north_west
                    .as_mut()
                    .unwrap()
                    .destruct(mesh_pool, root_entity, commands);
                self.south_east
                    .as_mut()
                    .unwrap()
                    .destruct(mesh_pool, root_entity, commands);
                self.south_west
                    .as_mut()
                    .unwrap()
                    .destruct(mesh_pool, root_entity, commands);
            }

            if self.entity.is_none() {
                self.entity =
                    Some(mesh_pool.get_mesh(commands, root_entity, mesh_cache, self.rect));
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
