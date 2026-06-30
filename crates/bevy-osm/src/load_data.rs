use std::{io::Read, path::Path};

use crate::{
    building::{polygon_building, spawn_building},
    cache::{
        cache_elevation_for_chunk, cache_raster_tile_for_chunk, cache_vector_tile_for_chunk,
        get_elevation_cache_path, get_elevation_cache_path_bevy, get_openfreemap_cache_path,
        get_osm_raster_cache_path, get_osm_raster_cache_path_bevy,
    },
    chunk::Chunk,
    config::OSMConfig,
    elevation::spawn_elevation_meshes,
    material::MapMaterialHandle,
    mesh::{BuildInstruction, LightInstruction, Shape, spawn_stroke_mesh},
    theme::get_way_build_instruction_openfreemap,
    vector::parse_pbf,
};
use bevy::{
    ecs::{system::SystemState, world::CommandQueue},
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future},
};
use bevy_terrain::quadtree::{ChunkLoaded, QuadTreeNodeComponent};
use lyon::geom::euclid::{Point2D, UnknownUnit};

#[derive(Component)]
pub struct ComputeTransform(pub Task<CommandQueue>);

#[derive(Component)]
pub struct ComputeVectorTile(pub Task<CommandQueue>);

pub fn preload_chunks(
    mut commands: Commands,
    nodes_to_load: Query<(Entity, &QuadTreeNodeComponent), Without<Chunk>>,
    config: Res<OSMConfig>,
) {
    nodes_to_load.iter().for_each(|(entity, node)| {
        let chunk = Chunk {
            x: node.x,
            y: node.y,
            z: node.lod as i8 + 9,
            elevation: Handle::default(),
            raster: Handle::default(),
        };
        cache_elevation_for_chunk(&chunk);
        cache_raster_tile_for_chunk(&chunk, &config);
        cache_vector_tile_for_chunk(&chunk);

        commands.entity(entity).insert(chunk);
    });
}

#[expect(clippy::too_many_arguments)]
pub fn load_unloaded_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map_materials: Res<MapMaterialHandle>,
    mut chunks_to_load: Query<(Entity, &mut Chunk), Without<ChunkLoaded>>,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    config: Res<OSMConfig>,
) {
    chunks_to_load.iter_mut().for_each(|(entity, mut chunk)| {
        let elevation_path_str = get_elevation_cache_path(&chunk);
        let elevation_path = Path::new(&elevation_path_str);
        let osm_raster_path_str = get_osm_raster_cache_path(&chunk, &config);
        let osm_raster_path = Path::new(&osm_raster_path_str);
        let vector_path_str = get_openfreemap_cache_path(&chunk);
        let vector_path = Path::new(&vector_path_str);

        if elevation_path.exists() && osm_raster_path.exists() && vector_path.exists() {
            chunk.elevation = asset_server.load(get_elevation_cache_path_bevy(&chunk));
            chunk.raster = asset_server.load(get_osm_raster_cache_path_bevy(&chunk, &config));

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

    // let vector_tile_chunk = match chunk.z > 14 {
    //     true => chunk.get_parent_at_z(14),
    //     false => chunk.clone(),
    // };

    let thread_pool = AsyncComputeTaskPool::get();

    // Spawn an async task to process the vector tile off the main thread.
    let building_material = map_materials.unknown_building.clone();
    let light_material = map_materials.light.clone();
    let vector_entity = commands.spawn_empty().id();
    let chunk_for_vector = chunk.clone();

    let vector_task = thread_pool.spawn(async move {
        let path = get_openfreemap_cache_path(&chunk_for_vector);
        let mut bytes = Vec::new();
        std::fs::File::open(&path)
            .expect("Vector tile file should exist")
            .read_to_end(&mut bytes)
            .expect("Could not read vector tile file");

        let instructions = parse_pbf(bytes).unwrap_or_default();

        let mut rng = rand::rng();
        let mut computed_strokes: Vec<Mesh> = Vec::new();
        let mut computed_buildings: Vec<Mesh> = Vec::new();
        let mut lights = Vec::new();

        for (tags, layer, polygon) in instructions {
            match get_way_build_instruction_openfreemap(tags, layer) {
                BuildInstruction::Stroke(stroke) => {
                    let points: Vec<Point2D<f32, UnknownUnit>> = polygon
                        .iter()
                        .map(|p| lyon::math::point(p.x, p.y))
                        .collect();

                    let center = points[0];
                    lights.push(LightInstruction {
                        trans: Vec3::new(center.x, 2.0, center.y),
                    });
                    computed_strokes.push(spawn_stroke_mesh(points, stroke));
                }
                BuildInstruction::Building(building_instr) => {
                    let building = polygon_building(&building_instr, polygon, &mut rng);
                    computed_buildings.push(spawn_building(&building));
                }
                _ => {}
            }
        }

        let mut merged_buildings: Vec<Mesh> = Vec::new();

        if !computed_buildings.is_empty() {
            let mut first = computed_buildings[0].clone();
            for other in computed_buildings.iter().skip(1) {
                first.merge(other).expect("could not merge buildings");
            }
            merged_buildings.push(first);
        }

        let light_transforms = lights
            .into_iter()
            .map(|light| Transform::from_translation(light.trans))
            .collect::<Vec<Transform>>();

        let mut command_queue = CommandQueue::default();
        command_queue.push(move |world: &mut World| {
            let mut meshes = SystemState::<ResMut<Assets<Mesh>>>::new(world).get_mut(world);

            let light_mesh = meshes.add(Cuboid::from_size(Vec3::new(0.003, 5.0, 0.003)));

            let stroke_handles: Vec<Handle<Mesh>> = computed_strokes
                .into_iter()
                .map(|m| meshes.add(m))
                .collect();

            let building_handles: Vec<Mesh3d> = merged_buildings
                .into_iter()
                .map(|m| Mesh3d(meshes.add(m)))
                .collect();

            for handle in stroke_handles {
                let stroke = world
                    .spawn((
                        Mesh3d(handle),
                        MeshMaterial3d(building_material.clone()),
                        Shape,
                    ))
                    .id();
                world.entity_mut(chunk_entity).add_child(stroke);
            }

            for mesh3d in building_handles {
                let bm = world
                    .spawn((
                        mesh3d,
                        MeshMaterial3d(building_material.clone()),
                        Transform::IDENTITY,
                    ))
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
            world
                .entity_mut(vector_entity)
                .remove::<ComputeVectorTile>();
        });
        command_queue
    });

    commands
        .entity(vector_entity)
        .insert(ComputeVectorTile(vector_task));

    spawn_elevation_meshes(
        commands,
        meshes,
        materials,
        heightmap,
        chunk_entity,
        chunk.clone(),
        config,
    );
}

pub fn handle_vector_tasks(
    mut commands: Commands,
    mut vector_tasks: Query<&mut ComputeVectorTile>,
) {
    for mut task in &mut vector_tasks {
        if let Some(mut commands_queue) = block_on(future::poll_once(&mut task.0)) {
            commands.append(&mut commands_queue);
        }
    }
}
