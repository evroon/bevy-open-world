use std::f32::consts::PI;

use bevy::prelude::*;

use crate::quadtree::{ChunkLoaded, DecreaseLOD, IncreaseLOD};

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
    let quadtree = QuadTree {};

    // Top
    // TODO: insert root:
    // QuadTreeNode::new(Vec2::ZERO, Vec2::splat(radius), 0, 0)
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

#[expect(clippy::type_complexity)]
pub fn update_terrain_quadtree(
    mut commands: Commands,
    camera: Single<&Transform, With<Camera>>,
    mut quadtrees: Query<(
        Entity,
        &mut QuadTree,
        &Children,
        &QuadTreeConfig,
        &Transform,
    )>,
    nodes: Query<(
        Entity,
        &mut QuadTreeNode,
        Option<&Children>,
        Option<&ChunkLoaded>,
        Option<&DecreaseLOD>,
        Option<&IncreaseLOD>,
    )>,
) {
    for (_, _, children, config, transform) in quadtrees.iter_mut() {
        assert!(children.len() == 1);
        nodes.get(children[0]).unwrap().1.build_around_point(
            config,
            children[0],
            &mut commands,
            &nodes,
            Vec3::new(
                -camera.translation.z,
                camera.translation.y,
                camera.translation.x,
            ) - transform.translation,
        );
    }
}
