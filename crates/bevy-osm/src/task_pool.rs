use crate::{
    building::spawn_building, location::Location, material::MapMaterialHandle, mesh::Shape,
    tile::build_tile,
};
use bevy::{
    ecs::{system::SystemState, world::CommandQueue},
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future},
};

#[derive(Component)]
pub struct ComputeTransform(pub Task<CommandQueue>);

pub fn spawn_task(mut commands: Commands, map_materials: Res<MapMaterialHandle>) {
    let thread_pool = AsyncComputeTaskPool::get();
    let building_material: Handle<StandardMaterial> = map_materials.unknown_building.clone();
    let light_material: Handle<StandardMaterial> = map_materials.light.clone();
    let entity = commands.spawn_empty().id();

    let task = thread_pool.spawn(async move {
        let (buildings, strokes, lights) = build_tile(Location::MonacoCenter);

        let mut command_queue = CommandQueue::default();

        command_queue.push(move |world: &mut World| {
            let mut meshes = SystemState::<ResMut<Assets<Mesh>>>::new(world).get_mut(world);
            let light_mesh = meshes.add(Cuboid::from_size(Vec3::splat(1.0)));

            let mut building_meshes = Vec::new();
            for building in buildings {
                let building = spawn_building(&building);
                for (mes, transform) in building {
                    building_meshes.push((Mesh3d(meshes.add(mes)), transform));
                }
            }

            let stroke_meshes: Vec<Handle<Mesh>> =
                strokes.iter().map(|s| meshes.add(s.clone())).collect();

            for mesh in stroke_meshes {
                world.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(building_material.clone()),
                    Shape,
                ));
            }

            for (mesh, trans) in building_meshes {
                world.spawn((mesh, MeshMaterial3d(building_material.clone()), trans));
            }

            for light in lights {
                world.spawn((
                    Mesh3d(light_mesh.clone()),
                    MeshMaterial3d(light_material.clone()),
                    Transform::from_translation(light.trans).with_scale(Vec3::splat(5.0)),
                ));
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
