use std::f32::consts::PI;

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
        root: QuadTreeNode::new(Vec2::ZERO, Vec2::splat(radius * 2.0)),
    };

    let grid = Grid::new(1.0e-1, 0.0);

    // Y+
    let (cell, pos) = planet_grid
        .grid()
        .translation_to_grid(Vec3::new(0.0, radius, 0.0));
    planet_grid.with_grid(grid.clone(), |quadtree_grid| {
        quadtree_grid.insert((
            QuadtreeGrid(),
            cell,
            Transform::from_translation(pos),
            quadtree.clone(),
            config.clone(),
            Visibility::Inherited,
            MeshPool::new(),
        ));
    });

    // Y-
    let (cell, pos) = planet_grid
        .grid()
        .translation_to_grid(Vec3::new(0.0, -radius, 0.0));
    planet_grid.with_grid(grid.clone(), |quadtree_grid| {
        quadtree_grid.insert((
            QuadtreeGrid(),
            cell,
            Transform::from_rotation(Quat::from_rotation_x(PI)).with_translation(pos),
            quadtree.clone(),
            config.clone(),
            Visibility::Inherited,
            MeshPool::new(),
        ));
    });
    // X+
    let (cell, pos) = planet_grid
        .grid()
        .translation_to_grid(Vec3::new(radius, 0.0, 0.0));
    planet_grid.with_grid(grid.clone(), |quadtree_grid| {
        quadtree_grid.insert((
            QuadtreeGrid(),
            cell,
            Transform::from_rotation(Quat::from_rotation_z(-PI * 0.5)).with_translation(pos),
            quadtree.clone(),
            config.clone(),
            Visibility::Inherited,
            MeshPool::new(),
        ));
    });
    // X-
    let (cell, pos) = planet_grid
        .grid()
        .translation_to_grid(Vec3::new(-radius, 0.0, 0.0));
    planet_grid.with_grid(grid.clone(), |quadtree_grid| {
        quadtree_grid.insert((
            QuadtreeGrid(),
            cell,
            Transform::from_rotation(Quat::from_rotation_z(PI * 0.5)).with_translation(pos),
            quadtree.clone(),
            config.clone(),
            Visibility::Inherited,
            MeshPool::new(),
        ));
    });
    // Z+
    let (cell, pos) = planet_grid
        .grid()
        .translation_to_grid(Vec3::new(0.0, 0.0, radius));
    planet_grid.with_grid(grid.clone(), |quadtree_grid| {
        quadtree_grid.insert((
            QuadtreeGrid(),
            cell,
            Transform::from_rotation(Quat::from_rotation_x(PI * 0.5)).with_translation(pos),
            quadtree.clone(),
            config.clone(),
            Visibility::Inherited,
            MeshPool::new(),
        ));
    });
    // Z-
    let (cell, pos) = planet_grid
        .grid()
        .translation_to_grid(Vec3::new(0.0, 0.0, -radius));
    planet_grid.with_grid(grid.clone(), |quadtree_grid| {
        quadtree_grid.insert((
            QuadtreeGrid(),
            cell,
            Transform::from_rotation(Quat::from_rotation_x(-PI * 0.5)).with_translation(pos),
            quadtree,
            config,
            Visibility::Inherited,
            MeshPool::new(),
        ));
    });
}

pub fn update_quadtree(
    mut commands: Commands,
    camera: Single<(&Transform, &CellCoord), With<Camera>>,
    mut quadtrees: Query<(
        Entity,
        &mut QuadTree,
        &Grid,
        &CellCoord,
        &QuadTreeConfig,
        &Transform,
        &mut MeshPool,
    )>,
    universe: Query<&Grid, With<UniverseGrid>>,
    mesh_cache: Res<MeshCache>,
) {
    if let Some(universe) = universe.iter().next() {
        let (cam_trans, cam_cell_coord) = *camera;

        for (entity, mut quadtree, grid, cell, config, grid_transform, mut mesh_pool) in
            quadtrees.iter_mut()
        {
            let grid_pos = (grid.grid_position(cam_cell_coord, cam_trans)
                - universe.grid_position(cell, grid_transform))
            .extend(1.0);
            // println!(
            //     "{} {}",
            //     (grid_transform.to_matrix().inverse() * grid_pos).xyz(),
            //     universe.grid_position(cell, grid_transform)
            // );
            quadtree.root.build_around_point(
                config,
                &mut mesh_pool,
                &mut commands,
                &mesh_cache,
                (grid_transform.to_matrix().inverse() * grid_pos).xyz(),
                &(entity, grid),
            );
        }
    } else {
        error!("Could not find universe grid");
    }
}
