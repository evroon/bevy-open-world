use std::f32::consts::PI;

use bevy::prelude::*;

use crate::quadtree::ChunkLoaded;

use super::quadtree::{QuadTree, QuadTreeConfig, QuadTreeNode};

pub fn build_planets(commands: Commands) {
    build_planet(commands, 40.0);
}

pub fn build_planet(mut commands: Commands, radius: f32) {
    let config = QuadTreeConfig {
        k: 1.1,
        max_lod: 24,
        min_lod: 2,
        size: radius,
    };
    let quadtree = QuadTree {
        root: QuadTreeNode::new(Vec2::ZERO, Vec2::splat(radius), 0, 0),
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
}

pub fn update_terrain_quadtree(
    mut commands: Commands,
    camera: Single<&Transform, With<Camera>>,
    mut quadtrees: Query<(Entity, &mut QuadTree, &QuadTreeConfig, &Transform)>,
    nodes_query: Query<(Entity, Option<&Children>, Option<&ChunkLoaded>)>,
) {
    for (entity, mut quadtree, config, transform) in quadtrees.iter_mut() {
        quadtree.root.build_around_point(
            config,
            &entity,
            &mut commands,
            camera.translation - transform.translation,
            &nodes_query,
        );
    }
}
