use std::f32::consts::PI;

use bevy::prelude::*;
use big_space::{grid::Grid, prelude::BigSpaceCommands};

use super::{
    mesh::MeshCache,
    quadtree::{MeshPool, QuadTree, QuadTreeConfig, QuadTreeNode},
};

pub fn build_planets(commands: Commands) {
    build_planet(commands, 40.0);
}

#[derive(Component)]
pub struct UniverseGrid();

#[derive(Component)]
pub struct QuadtreeGrid();

pub fn build_planet(mut commands: Commands, radius: f32) {
    commands.spawn(MeshPool::new());

    let config = QuadTreeConfig {
        k: 1.1,
        max_lod: 24,
        min_lod: 2,
        size: radius,
    };
    let quadtree = QuadTree {
        root: QuadTreeNode::new(Vec2::ZERO, Vec2::splat(radius)),
    };

    // Top
    commands.spawn((
        Transform::from_translation(Vec3::new(0.0, radius * 0.5, 0.0)),
        quadtree.clone(),
        config.clone(),
        Visibility::Inherited,
    ));
    // Bottom
    commands.spawn((
        Transform::from_rotation(Quat::from_rotation_x(PI)).with_translation(Vec3::new(
            0.0,
            -radius * 0.5,
            0.0,
        )),
        quadtree.clone(),
        config.clone(),
        Visibility::Inherited,
    ));
    // X+
    commands.spawn((
        Transform::from_rotation(Quat::from_rotation_z(-PI * 0.5)).with_translation(Vec3::new(
            radius * 0.5,
            0.0,
            0.0,
        )),
        quadtree.clone(),
        config.clone(),
        Visibility::Inherited,
    ));
    // X-
    commands.spawn((
        Transform::from_rotation(Quat::from_rotation_z(PI * 0.5)).with_translation(Vec3::new(
            -radius * 0.5,
            0.0,
            0.0,
        )),
        quadtree.clone(),
        config.clone(),
        Visibility::Inherited,
    ));
    // Z+
    commands.spawn((
        Transform::from_rotation(Quat::from_rotation_x(PI * 0.5)).with_translation(Vec3::new(
            0.0,
            0.0,
            radius * 0.5,
        )),
        quadtree.clone(),
        config.clone(),
        Visibility::Inherited,
    ));
    // Z-
    commands.spawn((
        Transform::from_rotation(Quat::from_rotation_x(-PI * 0.5)).with_translation(Vec3::new(
            0.0,
            0.0,
            -radius * 0.5,
        )),
        quadtree,
        config,
        Visibility::Inherited,
    ));

    commands.spawn_big_space_default(|root_grid| {
        root_grid.insert((QuadtreeGrid(),));
    });
}

pub fn update_quadtree(
    mut commands: Commands,
    camera: Single<&Transform, With<Camera>>,
    mut quadtrees: Query<(Entity, &mut QuadTree, &QuadTreeConfig, &Transform)>,
    root_grid: Single<(Entity, &Grid), With<QuadtreeGrid>>,
    mut mesh_pool: Single<&mut MeshPool>,
    mesh_cache: Res<MeshCache>,
) {
    let cam_trans = camera.translation;
    let norm = f32::abs(cam_trans.x)
        .max(f32::abs(cam_trans.y))
        .max(f32::abs(cam_trans.z));

    let cam_trans_dist_from_planet = cam_trans.length();

    let cam_translation_deformed_space = Vec4::new(
        camera.translation.x / norm * (cam_trans_dist_from_planet - 0.),
        camera.translation.y / norm * (cam_trans_dist_from_planet - 0.),
        camera.translation.z / norm * (cam_trans_dist_from_planet - 0.),
        1.0,
    );

    for (entity, mut quadtree, config, transform) in quadtrees.iter_mut() {
        quadtree.root.build_around_point(
            config,
            &entity,
            &mut mesh_pool,
            &mut commands,
            &mesh_cache,
            (transform.to_matrix().inverse() * cam_translation_deformed_space).xyz(),
            &root_grid,
        );
    }
}
