use bevy::{
    ecs::{system::SystemState, world::CommandQueue},
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future},
};

use crate::{
    clickhouse::{DataFetch, get_planes},
    config::ADSBConfig,
};

#[derive(Component)]
pub struct ComputeTransform(pub Task<CommandQueue>);

#[derive(Component)]
pub struct DataFetchReady;

pub fn spawn_task(mut commands: Commands, adsb: Res<ADSBConfig>, previous_data: Res<DataFetch>) {
    let thread_pool = AsyncComputeTaskPool::get();
    let entity = commands.spawn_empty().id();
    let buffer_time = adsb.buffer_time;
    let buffer_chunk_duration = adsb.buffer_chunk_duration;
    let existing_icaos = previous_data.icaos.clone();

    let task = thread_pool.spawn(async move {
        let mut command_queue = CommandQueue::default();

        let data_fetch_result = get_planes(
            buffer_time,
            buffer_time + buffer_chunk_duration,
            existing_icaos,
        )
        .await;

        command_queue.push(move |world: &mut World| {
            *SystemState::<ResMut<DataFetch>>::new(world).get_mut(world) = data_fetch_result;

            let mut manager = SystemState::<ResMut<ADSBConfig>>::new(world).get_mut(world);

            manager.buffer_time = manager.buffer_time + manager.buffer_chunk_duration;

            world.entity_mut(entity).remove::<ComputeTransform>();
        });

        command_queue
    });

    commands.entity(entity).insert(ComputeTransform(task));
}

pub fn handle_tasks(
    mut commands: Commands,
    adsb: Res<ADSBConfig>,
    mut transform_tasks: Query<&mut ComputeTransform>,
    previous_data: Res<DataFetch>,
) {
    for mut task in &mut transform_tasks {
        if let Some(mut commands_queue) = block_on(future::poll_once(&mut task.0)) {
            // append the returned command queue to have it execute later
            commands.append(&mut commands_queue);
        }
    }
    if transform_tasks.iter().len() == 0
        && adsb.time + adsb.buffer_ahead_duration > adsb.buffer_time
    {
        spawn_task(commands, adsb, previous_data);
    }
}
