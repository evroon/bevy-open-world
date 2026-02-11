//! Contains the plugin and its helper types.
//!
//! The [`Svg2dBundle`](crate::bundle::Svg2dBundle) provides a way to display an `SVG`-file
//! with minimal boilerplate.
//!
//! ## How it works
//! The user creates/loades a [`Svg2dBundle`](crate::bundle::Svg2dBundle) in a system.
//!
//! Then, in the [`Set::SVG`](Set::SVG), a mesh is created for each loaded [`Svg`] bundle.
//! Each mesh is then extracted in the [`RenderSet::Extract`](bevy::render::RenderSet) and added to the
//! [`RenderWorld`](bevy::render::RenderWorld).
//! Afterwards it is queued in the [`RenderSet::Queue`](bevy::render::RenderSet) for actual drawing/rendering.

use bevy::{
    app::{App, Plugin},
    asset::{AssetEvent, Assets},
    ecs::{
        entity::Entity,
        message::MessageReader,
        query::{Added, Changed, Or},
        schedule::{IntoScheduleConfigs, SystemSet},
        system::{Commands, Query, Res, ResMut},
    },
    log::debug,
    mesh::{Mesh, Mesh2d},
    prelude::{Last, PostUpdate},
};

use crate::{
    origin,
    render::{self, Svg2d},
    svg::Svg,
};

/// Set in which [`Svg`](crate::prelude::Svg2d)s get drawn.
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct SvgSet;

/// A plugin that makes sure your [`Svg`]s get rendered
pub struct SvgRenderPlugin;

impl Plugin for SvgRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, origin::add_origin_state.in_set(SvgSet))
            .add_systems(Last, (origin::apply_origin, svg_mesh_linker.in_set(SvgSet)))
            .add_plugins(render::SvgPlugin);
    }
}

/// Bevy system which queries for all [`Svg`] bundles and adds the correct [`Mesh`] to them.
fn svg_mesh_linker(
    mut commands: Commands,
    mut svg_events: MessageReader<AssetEvent<Svg>>,
    mut meshes: ResMut<Assets<Mesh>>,
    svgs: Res<Assets<Svg>>,
    mut query: Query<(Entity, Option<&'static Svg2d>, Option<&'static mut Mesh2d>)>,
    changed_handles: Query<Entity, Or<(Changed<Svg2d>, Added<Svg2d>)>>,
) {
    for event in svg_events.read() {
        match event {
            AssetEvent::Added { .. } => (),
            AssetEvent::LoadedWithDependencies { id } => {
                for (.., mesh_2d) in query
                    .iter_mut()
                    .filter(|(_, svg_2d, ..)| svg_2d.map(|x| x.0.id() == *id).unwrap_or(false))
                {
                    if let Some(mut mesh) = mesh_2d {
                        mesh.0 = svgs.get(*id).unwrap().mesh.clone();
                    }
                }
            }
            AssetEvent::Modified { id } => {
                for (.., mesh_2d) in query
                    .iter_mut()
                    .filter(|(_, svg_2d, _)| svg_2d.map(|x| x.0.id() == *id).unwrap_or(false))
                {
                    let svg = svgs.get(*id).unwrap();
                    if let Some(mut mesh) = mesh_2d.filter(|mesh| mesh.0 != svg.mesh) {
                        let old_mesh = mesh.0.clone();
                        mesh.0 = svg.mesh.clone();
                        meshes.remove(&old_mesh);
                    }
                }
            }
            AssetEvent::Removed { id } => {
                for (entity, ..) in query
                    .iter_mut()
                    .filter(|(_, svg_2d, _)| svg_2d.map(|x| x.0.id() == *id).unwrap_or(false))
                {
                    commands.entity(entity).despawn();
                }
            }
            AssetEvent::Unused { .. } => {}
        }
    }

    // Ensure all correct meshes are set for entities which have had modified handles
    for entity in changed_handles.iter() {
        let Ok((_, svg_2d, mesh_2d)) = query.get_mut(entity) else {
            continue;
        };
        let Some(handle) = svg_2d.map(|x| &x.0) else {
            continue;
        };
        let Some(svg) = svgs.get(handle) else {
            continue;
        };
        debug!(
            "Svg handle for entity `{:?}` modified. Changing mesh component of entity.",
            entity
        );
        if let Some(mut mesh) = mesh_2d {
            mesh.0 = svg.mesh.clone();
        }
    }
}
