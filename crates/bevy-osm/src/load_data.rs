use crate::{
    assets::mvt::MVTAsset,
    building::{build_building_mesh, polygon_building},
    cache::{load_elevation_for_chunk, load_raster_tile_for_chunk, load_vector_tile_for_chunk},
    chunk::Chunk,
    config::OSMConfig,
    elevation::{TILE_VERTEX_COUNT, get_elevation_local, spawn_elevation_meshes},
    material::{MapMaterialHandle, MapMeshHandle},
    mesh::{BuildInstruction, Shape, spawn_fill_mesh, spawn_stroke_mesh},
    theme::get_way_build_instruction_openfreemap,
};
use bevy::{
    ecs::{system::SystemState, world::CommandQueue},
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future},
};
use bevy_terrain::quadtree::{ChunkLoaded, QuadTreeNodeComponent};

#[derive(Component)]
pub struct ComputeTransform(pub Task<CommandQueue>);

#[derive(Component)]
pub struct ComputeVectorTile(pub Task<CommandQueue>);

#[derive(Component)]
pub struct OSMStroke;

#[derive(Component)]
pub struct OSMBuilding;

#[derive(Component)]
pub struct OSMLight;

pub fn insert_chunks_from_quadtree(
    mut commands: Commands,
    nodes_to_load: Query<(Entity, &QuadTreeNodeComponent), Without<Chunk>>,
    asset_server: Res<AssetServer>,
    config: Res<OSMConfig>,
) {
    nodes_to_load.iter().for_each(|(entity, node)| {
        let zxy = (node.lod + config.root_lod, node.x, node.y);
        let (z, x, y) = zxy;

        let vector = if z <= 14 {
            load_vector_tile_for_chunk((z, x, y), &asset_server)
        } else {
            let chunk = Chunk {
                z: z as i8,
                x,
                y,
                elevation: Handle::default(),
                raster: Handle::default(),
                vector: Handle::default(),
            };
            let parent_chunk = chunk.get_parent_at_z(14);
            load_vector_tile_for_chunk(
                (parent_chunk.z as u8, parent_chunk.x, parent_chunk.y),
                &asset_server,
            )
        };

        commands.entity(entity).insert(Chunk {
            z: z as i8,
            x,
            y,
            elevation: load_elevation_for_chunk(zxy, &asset_server),
            raster: load_raster_tile_for_chunk(zxy, &config, &asset_server),
            vector,
        });
    });
}

#[expect(clippy::too_many_arguments)]
pub fn load_unloaded_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map_materials: Res<MapMaterialHandle>,
    map_meshes: Res<MapMeshHandle>,
    mut chunks_to_load: Query<(Entity, &Chunk), Without<ChunkLoaded>>,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    mvt_assets: Res<Assets<MVTAsset>>,
    config: Res<OSMConfig>,
) {
    chunks_to_load.iter_mut().for_each(|(entity, chunk)| {
        if asset_server.is_loaded(chunk.elevation.id())
            && asset_server.is_loaded(chunk.raster.id())
            && asset_server.is_loaded(chunk.vector.id())
        {
            load_chunk(
                &mut commands,
                &mut meshes,
                &mut materials,
                &map_materials,
                &map_meshes,
                &images,
                &mvt_assets,
                &config,
                entity,
                chunk.clone(),
            );
        }
    });
}
pub fn get_elevation(translation: Vec3, heightmap: &Image) -> Option<f32> {
    if !Rect::from_center_size(Vec2::ZERO, Vec2::ONE).contains(translation.xz()) {
        return None;
    }
    let local_coords =
        ((Vec2::new(0.5, 0.5) + translation.xz()) * TILE_VERTEX_COUNT as f32).as_ivec2();
    Some(get_elevation_local(heightmap, local_coords))
}

#[expect(clippy::too_many_arguments)]
pub fn load_chunk(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    map_materials: &Res<MapMaterialHandle>,
    map_meshes: &Res<MapMeshHandle>,
    images: &Res<Assets<Image>>,
    mvt_assets: &Res<Assets<MVTAsset>>,
    config: &Res<OSMConfig>,
    chunk_entity: Entity,
    chunk: Chunk,
) {
    let elevation = chunk.elevation.id();
    let heightmap = images
        .get(elevation)
        .expect("Image should have loaded by now")
        .clone();

    let vector = chunk.vector.id();
    let mvt = mvt_assets
        .get(vector)
        .expect("MVT should have loaded by now")
        .instructions
        .clone();

    let thread_pool = AsyncComputeTaskPool::get();
    let building_material = map_materials.unknown_building.clone();
    let light_materials = map_materials.lights.clone();
    let light_mesh = map_meshes.light.clone();
    let strokes_visible = config.strokes_visible;
    let buildings_visible = config.buildings_visible;
    let lights_visible = config.lights_visible;
    let vector_entity = commands.spawn_empty().id();

    let value = chunk.clone();

    let vector_task = thread_pool.spawn(async move {
        let mut computed_meshes: Vec<Mesh> = Vec::new();
        let mut computed_buildings: Vec<Mesh> = Vec::new();
        let mut lights: Vec<crate::mesh::LightInstruction> = Vec::new();
        let mut affine = (Vec3::ONE, Vec3::ONE);
        let mut rect = Rect::default();

        if chunk.z > 14 {
            let parent_chunk = value.get_parent_at_z(14);
            rect = value.get_rect_inside_parent(&parent_chunk);
            affine = value.get_affine_from_child_to_parent(&parent_chunk);
        }

        for (tags, layer, polygon) in mvt {
            for a in get_way_build_instruction_openfreemap(tags, layer, polygon, &heightmap) {
                match a {
                    BuildInstruction::Light(light) => {
                        if chunk.z > 14 {
                            if rect.contains(light.trans.xz()) {
                                let mut light = light.clone();
                                light.trans += affine.1;
                                light.trans *= affine.0;
                                lights.push(light);
                            }
                        } else {
                            lights.push(light);
                        }
                    }
                    // BuildInstruction::Fill(fill) if fill.polygon.iter().len() > 2 => {
                    //     computed_meshes.push(spawn_fill_mesh(fill.polygon.clone(), fill));
                    // }
                    // BuildInstruction::Stroke(stroke) if stroke.polygon.iter().len() > 2 => {
                    //     computed_meshes.push(spawn_stroke_mesh(stroke.polygon.clone(), stroke));
                    // }
                    BuildInstruction::Building(building) => {
                        let building_polygon =
                            polygon_building(&building, building.polygon.clone());
                        let mesh = build_building_mesh(&building_polygon).translated_by(
                            Vec3::Y
                                * (get_elevation(building_polygon.get_translation(), &heightmap)
                                    .unwrap_or(0.0)
                                    + building.offset_above_ground_level),
                        );

                        if chunk.z > 14 {
                            if rect.contains(building_polygon.get_translation().xz()) {
                                computed_buildings
                                    .push(mesh.translated_by(affine.1).scaled_by(affine.0));
                            }
                        } else {
                            computed_buildings.push(mesh);
                        }
                    }
                    _ => {}
                }
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

        let mut command_queue = CommandQueue::default();
        command_queue.push(move |world: &mut World| {
            // If the chunk was despawned while the async task was running, discard
            // the results and clean up the task entity to avoid a panic.
            if world.get_entity(chunk_entity).is_err() {
                if let Ok(ve) = world.get_entity_mut(vector_entity) {
                    ve.despawn();
                }
                return;
            }

            let mut meshes = SystemState::<ResMut<Assets<Mesh>>>::new(world)
                .get_mut(world)
                .unwrap();

            let stroke_handles: Vec<Handle<Mesh>> =
                computed_meshes.into_iter().map(|m| meshes.add(m)).collect();

            let building_handles: Vec<Mesh3d> = merged_buildings
                .into_iter()
                .map(|m| Mesh3d(meshes.add(m)))
                .collect();

            for handle in stroke_handles {
                let stroke = world
                    .spawn((
                        Mesh3d(handle),
                        MeshMaterial3d(building_material.clone()),
                        if strokes_visible {
                            Visibility::Visible
                        } else {
                            Visibility::Hidden
                        },
                        Shape,
                        OSMStroke,
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
                        if buildings_visible {
                            Visibility::Visible
                        } else {
                            Visibility::Hidden
                        },
                        OSMBuilding,
                    ))
                    .id();
                world.entity_mut(chunk_entity).add_child(bm);
            }

            for light in lights {
                let l = world
                    .spawn((
                        Mesh3d(light_mesh.clone()),
                        MeshMaterial3d(light_materials[&light.color].clone()),
                        Transform::from_translation(light.trans),
                        if lights_visible {
                            Visibility::Visible
                        } else {
                            Visibility::Hidden
                        },
                        OSMLight,
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
    mut vector_tasks: Query<&mut ComputeVectorTile>,
) {
    for mut task in &mut vector_tasks {
        if let Some(mut commands_queue) = block_on(future::poll_once(&mut task.0)) {
            commands.append(&mut commands_queue);
        }
    }
}
