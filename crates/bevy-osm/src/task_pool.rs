use crate::{
    building::spawn_building,
    chunk::{Chunk, ChunkLoaded},
    elevation::{
        TILE_VERTEX_COUNT, cache_elevation_for_chunk, cache_raster_tile_for_chunk,
        get_elevation_local, spawn_elevation_meshes,
    },
    material::MapMaterialHandle,
    mesh::Shape,
    tile::build_tile,
};
use bevy::{
    ecs::{system::SystemState, world::CommandQueue},
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future},
};

#[derive(Component)]
pub struct ComputeTransform(pub Task<CommandQueue>);

pub fn preload_chunk(mut commands: Commands, asset_server: Res<AssetServer>, mut chunk: Chunk) {
    cache_elevation_for_chunk(chunk.clone());
    cache_raster_tile_for_chunk(chunk.clone());

    chunk.elevation = asset_server.load(chunk.get_elevation_cache_path_bevy());
    chunk.raster = asset_server.load(chunk.get_osm_raster_cache_path_bevy());

    commands.spawn(chunk.clone());
}

pub fn load_unloaded_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map_materials: Res<MapMaterialHandle>,
    chunks_to_load: Query<(Entity, &Chunk), Without<ChunkLoaded>>,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
) {
    chunks_to_load.iter().for_each(|(entity, chunk)| {
        if asset_server.is_loaded(chunk.elevation.id()) {
            load_chunk(
                &mut commands,
                &mut meshes,
                &mut materials,
                &map_materials,
                &images,
                entity,
                chunk.clone(),
            )
        }
    });
}

pub fn load_chunk(
    commands: &mut Commands,
    meshes2: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    map_materials: &Res<MapMaterialHandle>,
    images: &Res<Assets<Image>>,
    entity: Entity,
    chunk: Chunk,
) {
    let elevation = chunk.elevation.id();
    let image = images
        .get(elevation)
        .expect("Image should have loaded by now");

    spawn_elevation_meshes(commands, meshes2, materials, image, entity, chunk.clone());

    let thread_pool = AsyncComputeTaskPool::get();
    let building_material: Handle<StandardMaterial> = map_materials.unknown_building.clone();
    let light_material: Handle<StandardMaterial> = map_materials.light.clone();
    let entity = commands.spawn_empty().id();
    let area = chunk.get_lat_lon_area();
    let size_meters = area.size() * chunk.lat_lon_to_meters();

    let task = thread_pool.spawn(async move {
        let (buildings, strokes, lights) = build_tile(chunk);

        let mut command_queue = CommandQueue::default();

        command_queue.push(move |world: &mut World| {
            let images = SystemState::<Res<Assets<Image>>>::new(world).get(world);
            let image = images
                .get(elevation)
                .expect("Image should have loaded by now");

            let building_meshes = buildings
                .iter()
                .flat_map(|building| {
                    spawn_building(building)
                        .into_iter()
                        .map(|(mesh, mut transform)| {
                            let translation = transform.translation;
                            let elevation = get_elevation_local(
                                image,
                                ((translation.x / size_meters.y + 0.5) * TILE_VERTEX_COUNT as f32)
                                    as i32,
                                ((translation.z / size_meters.x + 0.5) * TILE_VERTEX_COUNT as f32)
                                    as i32,
                            );
                            transform.translation += Vec3::Y * elevation;
                            (mesh, transform)
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<(Mesh, Transform)>>();

            let mut meshes = SystemState::<ResMut<Assets<Mesh>>>::new(world).get_mut(world);

            let mesh3ds = building_meshes
                .into_iter()
                .map(|(bm, t)| (Mesh3d(meshes.add(bm)), t))
                .collect::<Vec<(Mesh3d, Transform)>>();

            let light_mesh = meshes.add(Cuboid::from_size(Vec3::splat(1.0)));

            let stroke_meshes: Vec<Handle<Mesh>> =
                strokes.iter().map(|s| meshes.add(s.clone())).collect();

            for mesh in stroke_meshes {
                world.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(building_material.clone()),
                    Shape,
                ));
            }

            for (mesh, trans) in mesh3ds {
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
