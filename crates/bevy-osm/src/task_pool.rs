//! This example shows how to use the ECS and the [`AsyncComputeTaskPool`]
//! to spawn, poll, and complete tasks across systems and system ticks.

use bevy::{
    ecs::{system::SystemState, world::CommandQueue},
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future},
};

use crate::{build_tile, building::spawn_building, material::MapMaterialHandle};

#[derive(Component)]
pub struct ComputeTransform(pub Task<CommandQueue>);

pub fn spawn_task(mut commands: Commands, map_materials: Res<MapMaterialHandle>) {
    let thread_pool = AsyncComputeTaskPool::get();
    let handle: Handle<StandardMaterial> = map_materials.unknown_building.clone();
    let entity = commands.spawn_empty().id();
    let task = thread_pool.spawn(async move {
        let buildings = build_tile();

        let mut command_queue = CommandQueue::default();

        command_queue.push(move |world: &mut World| {
            let mut meshes = SystemState::<ResMut<Assets<Mesh>>>::new(world).get_mut(world);

            let mut mt = Vec::new();
            for building in buildings {
                let a = spawn_building(&building);
                for (mes, transform) in a {
                    mt.push((Mesh3d(meshes.add(mes)), transform));
                }
            }

            for (mesh, trans) in mt {
                world.spawn((mesh, MeshMaterial3d(handle.clone()), trans));
            }

            world.entity_mut(entity).remove::<ComputeTransform>();
        });

        command_queue
    });

    commands.entity(entity).insert(ComputeTransform(task));
}

pub fn handle_tasks(mut commands: Commands, mut transform_tasks: Query<&mut ComputeTransform>) {
    for mut task in &mut transform_tasks {
        if let Some(mut commands_queue) = block_on(future::poll_once(&mut task.0)) {
            // append the returned command queue to have it execute later
            commands.append(&mut commands_queue);
        }
    }
}
