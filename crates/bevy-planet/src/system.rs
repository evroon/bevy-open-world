use bevy::prelude::*;
use big_space::{
    grid::Grid,
    prelude::{CellCoord, GridCommands},
};

use super::{
    mesh::MeshCache,
    quadtree::{MeshPool, QuadTree, QuadTreeConfig, QuadTreeNode},
};

#[derive(Component)]
pub struct UniverseGrid();

#[derive(Component)]
pub struct PlanetGrid();

#[derive(Component)]
pub struct QuadtreeGrid();

pub fn build_planet(planet_grid: &mut GridCommands, radius: f32) {
    planet_grid.spawn_spatial(MeshPool::new());

    let config = QuadTreeConfig {
        k: 1.1,
        max_lod: 24,
        min_lod: 2,
        size: radius,
    };
    let quadtree = QuadTree {
        root: QuadTreeNode::new(Vec2::ZERO, Vec2::splat(radius)),
    };

    // Y+
    let (cell, pos) = planet_grid
        .grid()
        .translation_to_grid(Vec3::new(0.0, radius * 0.0, 0.0));
    // planet_grid.with_grid(Grid::new(1.0e-2f32, 0.0), |quadtree_grid| {
    planet_grid.insert((
        QuadtreeGrid(),
        cell,
        Transform::from_translation(pos),
        quadtree.clone(),
        config.clone(),
        Visibility::Inherited,
        MeshPool::new(),
    ));
    // });
    // // Y-
    // let (cell, pos) = planet_grid
    //     .grid()
    //     .translation_to_grid(Vec3::new(0.0, radius * 0.5, 0.0));
    // planet_grid.with_grid_default(|quadtree_grid| {
    //     quadtree_grid.insert((
    //         QuadtreeGrid(),
    //         cell,
    //         Transform::from_rotation(Quat::from_rotation_x(PI)).with_translation(pos),
    //         quadtree.clone(),
    //         config.clone(),
    //         Visibility::Inherited,
    //         MeshPool::new(),
    //     ));
    // });
    // // X+
    // let (cell, pos) = planet_grid
    //     .grid()
    //     .translation_to_grid(Vec3::new(0.0, radius * 0.5, 0.0));
    // planet_grid.with_grid_default(|quadtree_grid| {
    //     quadtree_grid.insert((
    //         QuadtreeGrid(),
    //         cell,
    //         Transform::from_rotation(Quat::from_rotation_z(-PI * 0.5)).with_translation(pos),
    //         quadtree.clone(),
    //         config.clone(),
    //         Visibility::Inherited,
    //         MeshPool::new(),
    //     ));
    // });
    // // X-
    // let (cell, pos) = planet_grid
    //     .grid()
    //     .translation_to_grid(Vec3::new(0.0, radius * 0.5, 0.0));
    // planet_grid.with_grid_default(|quadtree_grid| {
    //     quadtree_grid.insert((
    //         QuadtreeGrid(),
    //         cell,
    //         Transform::from_rotation(Quat::from_rotation_z(PI * 0.5)).with_translation(pos),
    //         quadtree.clone(),
    //         config.clone(),
    //         Visibility::Inherited,
    //         MeshPool::new(),
    //     ));
    // });
    // // Z+
    // let (cell, pos) = planet_grid
    //     .grid()
    //     .translation_to_grid(Vec3::new(0.0, radius * 0.5, 0.0));
    // planet_grid.with_grid_default(|quadtree_grid| {
    //     quadtree_grid.insert((
    //         QuadtreeGrid(),
    //         cell,
    //         Transform::from_rotation(Quat::from_rotation_x(PI * 0.5)).with_translation(pos),
    //         quadtree.clone(),
    //         config.clone(),
    //         Visibility::Inherited,
    //         MeshPool::new(),
    //     ));
    // });
    // // Z-
    // let (cell, pos) = planet_grid
    //     .grid()
    //     .translation_to_grid(Vec3::new(0.0, radius * 0.5, 0.0));
    // planet_grid.with_grid_default(|quadtree_grid| {
    //     quadtree_grid.insert((
    //         QuadtreeGrid(),
    //         cell,
    //         Transform::from_rotation(Quat::from_rotation_x(-PI * 0.5)).with_translation(pos),
    //         quadtree,
    //         config,
    //         Visibility::Inherited,
    //         MeshPool::new(),
    //     ));
    // });
}

pub fn update_quadtree(
    mut commands: Commands,
    camera: Single<(&Transform, &CellCoord), With<Camera>>,
    mut quadtrees: Query<(
        Entity,
        &mut QuadTree,
        &Grid,
        &QuadTreeConfig,
        &Transform,
        &GlobalTransform,
        &mut MeshPool,
    )>,
    mesh_cache: Res<MeshCache>,
) {
    let (cam_trans, cam_cell_coord) = *camera;
    // let norm = f32::abs(cam_trans.x)
    //     .max(f32::abs(cam_trans.y))
    //     .max(f32::abs(cam_trans.z));

    // let cam_trans_dist_from_planet = cam_trans.length();

    // let cam_translation_deformed_space = Vec4::new(
    //     camera.translation.x / norm * (cam_trans_dist_from_planet - 0.0),
    //     camera.translation.y / norm * (cam_trans_dist_from_planet - 0.0),
    //     camera.translation.z / norm * (cam_trans_dist_from_planet - 0.0),
    //     1.0,
    // );

    for (entity, mut quadtree, grid, config, transform, _, mut mesh_pool) in quadtrees.iter_mut() {
        // println!("{:?}", transform.translation);
        quadtree.root.build_around_point(
            config,
            &mut mesh_pool,
            &mut commands,
            &mesh_cache,
            // (transform.to_matrix().inverse() * cam_translation_deformed_space).xyz(),
            grid.grid_position(cam_cell_coord, cam_trans) - transform.translation,
            &(entity, grid),
        );
    }
}
