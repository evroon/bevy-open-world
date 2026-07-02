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
    elevation::{TILE_VERTEX_COUNT, get_elevation_local, spawn_elevation_meshes},
    material::{MapMaterialHandle, MapMeshHandle},
    mesh::{BuildInstruction, LightInstruction, Shape, spawn_stroke_mesh},
    theme::get_way_build_instruction_openfreemap,
    vector::parse_pbf,
};
use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future},
};
use bevy_terrain::quadtree::{ChunkLoaded, QuadTreeNodeComponent};

#[derive(Component)]
pub struct ComputeTransform(pub Task<()>);

/// Data produced by the async vector-tile compute task.
pub struct VectorTileResult {
    pub chunk_entity: Entity,
    pub computed_strokes: Vec<Mesh>,
    pub merged_buildings: Vec<Mesh>,
    pub light_transforms: Vec<Transform>,
}

#[derive(Component)]
pub struct ComputeVectorTile(pub Task<VectorTileResult>);

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

pub fn load_unloaded_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
                    &images,
                    &config,
                    entity,
                    chunk.clone(),
                )
            }
        }
    });
}

pub fn load_chunk(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    images: &Res<Assets<Image>>,
    config: &Res<OSMConfig>,
    chunk_entity: Entity,
    chunk: Chunk,
) {
    let elevation = chunk.elevation.id();
    let heightmap = images
        .get(elevation)
        .expect("Image should have loaded by now")
        .clone();

    // let vector_tile_chunk = match chunk.z > 14 {
    //     true => chunk.get_parent_at_z(14),
    //     false => chunk.clone(),
    // };

    let thread_pool = AsyncComputeTaskPool::get();
    let get_elevation = move |translation: Vec3, heightmap: &Image| {
        if !Rect::from_center_size(Vec2::ZERO, Vec2::ONE).contains(translation.xz()) {
            return None;
        }
        let local_coords =
            ((Vec2::new(0.5, 0.5) + translation.xz()) * TILE_VERTEX_COUNT as f32).as_ivec2();
        Some(get_elevation_local(heightmap, local_coords))
    };

    // Spawn an async task to process the vector tile off the main thread.
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
                    let center = polygon[0];
                    lights.push(LightInstruction {
                        trans: Vec3::new(
                            center.x,
                            get_elevation(Vec3::new(center.x, 0.0, center.y), &heightmap)
                                .unwrap_or(0.0)
                                + 2.0,
                            center.y,
                        ),
                    });
                    computed_strokes.push(spawn_stroke_mesh(polygon, stroke));
                }
                BuildInstruction::Building(building_instr) => {
                    let building = polygon_building(&building_instr, polygon, &mut rng);
                    let mesh = spawn_building(&building);
                    computed_buildings.push(mesh.translated_by(
                        Vec3::Y
                            * get_elevation(building.get_translation(), &heightmap).unwrap_or(0.0),
                    ));
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

        VectorTileResult {
            chunk_entity,
            computed_strokes,
            merged_buildings,
            light_transforms,
        }
    });

    commands
        .entity(vector_entity)
        .insert(ComputeVectorTile(vector_task));

    spawn_elevation_meshes(
        commands,
        meshes,
        materials,
        &images
            .get(elevation)
            .expect("Image should have loaded by now")
            .clone(),
        chunk_entity,
        chunk.clone(),
        config,
    );
}

pub fn handle_vector_tasks(
    mut commands: Commands,
    mut vector_tasks: Query<(Entity, &mut ComputeVectorTile)>,
    mut meshes: ResMut<Assets<Mesh>>,
    map_materials: Res<MapMaterialHandle>,
    map_meshes: Res<MapMeshHandle>,
    chunk_query: Query<Entity, With<Chunk>>,
) {
    for (vector_entity, mut task) in &mut vector_tasks {
        if let Some(result) = block_on(future::poll_once(&mut task.0)) {
            // If the chunk was despawned while the async task was running, discard
            // the results and clean up the task entity to avoid a panic.
            if chunk_query.get(result.chunk_entity).is_err() {
                commands.entity(vector_entity).despawn();
                continue;
            }

            let stroke_handles: Vec<Handle<Mesh>> = result
                .computed_strokes
                .into_iter()
                .map(|m| meshes.add(m))
                .collect();

            let building_handles: Vec<Mesh3d> = result
                .merged_buildings
                .into_iter()
                .map(|m| Mesh3d(meshes.add(m)))
                .collect();

            for handle in stroke_handles {
                let stroke = commands
                    .spawn((
                        Mesh3d(handle),
                        MeshMaterial3d(map_materials.unknown_building.clone()),
                        Shape,
                    ))
                    .id();
                commands.entity(result.chunk_entity).add_child(stroke);
            }

            for mesh3d in building_handles {
                let bm = commands
                    .spawn((
                        mesh3d,
                        MeshMaterial3d(map_materials.unknown_building.clone()),
                        Transform::IDENTITY,
                    ))
                    .id();
                commands.entity(result.chunk_entity).add_child(bm);
            }

            for transform in result.light_transforms {
                let l = commands
                    .spawn((
                        Mesh3d(map_meshes.light.clone()),
                        MeshMaterial3d(map_materials.light.clone()),
                        transform,
                    ))
                    .id();
                commands.entity(result.chunk_entity).add_child(l);
            }

            commands.entity(vector_entity).remove::<ComputeVectorTile>();
        }
    }
}
