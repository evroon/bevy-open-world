use crate::{
    building::spawn_building,
    chunk::{Chunk, ChunkLoaded},
    config::OSMConfig,
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
use bevy_terrain::quadtree::QuadTreeNodeComponent;

#[derive(Component)]
pub struct ComputeTransform(pub Task<CommandQueue>);

pub fn preload_chunks(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    nodes_to_load: Query<(Entity, &QuadTreeNodeComponent), Without<Chunk>>,
) {
    nodes_to_load.iter().for_each(|(entity, node)| {
        let mut chunk = Chunk {
            x: node.x,
            y: node.y,
            z: node.lod as i8 + 11,
            elevation: Handle::default(),
            raster: Handle::default(),
        };
        cache_elevation_for_chunk(chunk.clone());
        cache_raster_tile_for_chunk(chunk.clone());

        chunk.elevation = asset_server.load(chunk.get_elevation_cache_path_bevy());
        chunk.raster = asset_server.load(chunk.get_osm_raster_cache_path_bevy());

        commands.entity(entity).insert(chunk);
    });
}

#[expect(clippy::too_many_arguments)]
pub fn load_unloaded_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map_materials: Res<MapMaterialHandle>,
    chunks_to_load: Query<(Entity, &Chunk), Without<ChunkLoaded>>,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    config: Res<OSMConfig>,
) {
    chunks_to_load.iter().for_each(|(entity, chunk)| {
        if asset_server.is_loaded(chunk.elevation.id()) {
            load_chunk(
                &mut commands,
                &mut meshes,
                &mut materials,
                &map_materials,
                &images,
                &config,
                entity,
                chunk.clone(),
            )
        }
    });
}

#[expect(clippy::too_many_arguments)]
pub fn load_chunk(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    map_materials: &Res<MapMaterialHandle>,
    images: &Res<Assets<Image>>,
    config: &Res<OSMConfig>,
    chunk_entity: Entity,
    chunk: Chunk,
) {
    let elevation = chunk.elevation.id();
    let heightmap = images
        .get(elevation)
        .expect("Image should have loaded by now");

    spawn_elevation_meshes(
        commands,
        meshes,
        materials,
        heightmap,
        chunk_entity,
        chunk.clone(),
        config,
    );

    let thread_pool = AsyncComputeTaskPool::get();
    let building_material: Handle<StandardMaterial> = map_materials.unknown_building.clone();
    let light_material: Handle<StandardMaterial> = map_materials.light.clone();
    let entity = commands.spawn_empty().id();
    let lat_lon_origin = config.location.get_world_center();
    let area_meters = chunk.get_area_in_meters(config.location.get_world_center());
    let origin_meters = area_meters.center();
    let size_meters = area_meters.size();

    let get_elevation = move |translation: Vec3, heightmap: &Image| {
        if !area_meters.contains(translation.xz()) {
            return None;
        }
        let local_coords = (((translation.xz() - origin_meters) / size_meters + Vec2::splat(0.5))
            * TILE_VERTEX_COUNT as f32)
            .as_ivec2();
        Some(get_elevation_local(heightmap, local_coords))
    };

    let _task = thread_pool.spawn(async move {
        let (buildings, strokes, lights) = build_tile(chunk, lat_lon_origin);

        let mut command_queue = CommandQueue::default();

        command_queue.push(move |world: &mut World| {
            let images = SystemState::<Res<Assets<Image>>>::new(world).get(world);
            let heightmap = images
                .get(elevation)
                .expect("Image should have loaded by now");

            let building_meshes = buildings
                .iter()
                .flat_map(|building| {
                    spawn_building(building)
                        .into_iter()
                        .filter_map(|(mesh, mut transform)| {
                            let translation = transform.translation;
                            if let Some(elevation) = get_elevation(translation, heightmap) {
                                transform.translation += Vec3::Y * elevation;
                                return Some((mesh, transform));
                            }
                            None
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<(Mesh, Transform)>>();

            let light_transforms = lights
                .into_iter()
                .filter_map(|light| {
                    let mut translation = light.trans;
                    if let Some(elevation) = get_elevation(translation, heightmap) {
                        translation += Vec3::Y * elevation;
                        return Some(
                            Transform::from_translation(translation).with_scale(Vec3::splat(5.0)),
                        );
                    }
                    None
                })
                .collect::<Vec<Transform>>();

            let mut meshes = SystemState::<ResMut<Assets<Mesh>>>::new(world).get_mut(world);

            let mesh3ds = building_meshes
                .into_iter()
                .map(|(mesh, t)| (Mesh3d(meshes.add(mesh)), t))
                .collect::<Vec<(Mesh3d, Transform)>>();

            let light_mesh = meshes.add(Cuboid::from_size(Vec3::splat(1.0)));

            let stroke_meshes: Vec<Handle<Mesh>> =
                strokes.iter().map(|s| meshes.add(s.clone())).collect();

            for mesh in stroke_meshes {
                let stroke = world
                    .spawn((
                        Mesh3d(mesh),
                        MeshMaterial3d(building_material.clone()),
                        Shape,
                    ))
                    .id();
                world.entity_mut(chunk_entity).add_child(stroke);
            }

            for (mesh, trans) in mesh3ds {
                let bm = world
                    .spawn((mesh, MeshMaterial3d(building_material.clone()), trans))
                    .id();
                world.entity_mut(chunk_entity).add_child(bm);
            }

            for transform in light_transforms {
                let l = world
                    .spawn((
                        Mesh3d(light_mesh.clone()),
                        MeshMaterial3d(light_material.clone()),
                        transform,
                    ))
                    .id();
                world.entity_mut(chunk_entity).add_child(l);
            }

            world.entity_mut(entity).remove::<ComputeTransform>();
        });

        command_queue
    });

    // commands.entity(entity).insert(ComputeTransform(task));
}

pub fn handle_tasks(mut commands: Commands, mut transform_tasks: Query<&mut ComputeTransform>) {
    for mut task in &mut transform_tasks {
        if let Some(mut commands_queue) = block_on(future::poll_once(&mut task.0)) {
            // append the returned command queue to have it execute later
            commands.append(&mut commands_queue);
        }
    }
}
