use std::{fs::File, io::Write, path::Path};

use bevy::log::info;
use bevy_terrain::mesh::{HeightMap, build_mesh_data, iterate_mesh_vertices};

use crate::{
    chunk::{Chunk, ChunkLoaded},
    config::OSMConfig,
};
use bevy::prelude::*;

const ELEVATION_BASE_URL: &str = "https://tiles.mapterhorn.com";
const RASTER_BASE_URL: &str = "https://tile.openstreetmap.org";
const HEIGHT_OFFSET: f32 = 130.0;

pub fn cache_elevation_for_chunk(chunk: Chunk) {
    chunk.ensure_cache_dirs_exist();

    let path_str = chunk.get_elevation_cache_path();
    let path = Path::new(&path_str);

    if !path.exists() {
        info!("Downloading elevation tile for {chunk:?}");

        let (z, x, y) = (chunk.z, chunk.x, chunk.y);
        let request = ehttp::Request::get(format!("{ELEVATION_BASE_URL}/{z}/{x}/{y}.webp"));

        File::create(path)
            .unwrap()
            .write_all(&ehttp::fetch_blocking(&request).unwrap().bytes)
            .expect("Could not write to tile cache");

        info!("Finished downloading elevation tile for {chunk:?}");
    }
}

pub fn cache_raster_tile_for_chunk(chunk: Chunk) {
    chunk.ensure_cache_dirs_exist();

    let path_str = chunk.get_osm_raster_cache_path();
    let path = Path::new(&path_str);

    if !path.exists() {
        info!("Downloading raster tile for {chunk:?}");

        let (z, x, y) = (chunk.z, chunk.x, chunk.y);
        let request = ehttp::Request::get(format!("{RASTER_BASE_URL}/{z}/{x}/{y}.png"));

        File::create(path)
            .unwrap()
            .write_all(&ehttp::fetch_blocking(&request).unwrap().bytes)
            .expect("Could not write to tile cache");

        info!("Finished downloading raster tile for {chunk:?}");
    }
}

fn elevation_color_to_height_meters(c: Color) -> f32 {
    let lin_color = c.to_srgba();
    (lin_color.red * 256.0 * 256.0 + lin_color.green * 256.0 + lin_color.blue)
        - 32768.0
        - HEIGHT_OFFSET
}

pub fn spawn_elevation_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    chunks_to_load: Query<(Entity, &Chunk), Without<ChunkLoaded>>,
) {
    let vertex_count = 512;

    chunks_to_load.iter().for_each(|(entity, chunk)| {
        if asset_server.is_loaded(chunk.elevation.id()) {
            let image = images
                .get(chunk.elevation.id())
                .expect("Image should have loaded by now");

            let world_rect = chunk.get_lat_lon_area();
            let chunk_lat_lon_to_meters = chunk.lat_lon_to_meters();
            let size_meters = world_rect.size() * chunk_lat_lon_to_meters;

            info!("Terrain size in meters: {size_meters:?}");
            info!("Terrain size in lat, lon: {world_rect:?}");

            let heights = iterate_mesh_vertices(IVec2::splat(vertex_count), world_rect)
                .map(|(x_local, y_local, ..)| {
                    (
                        (x_local, y_local),
                        elevation_color_to_height_meters(
                            image
                                .get_color_at(
                                    // Clamp to border
                                    x_local.max(0).min(vertex_count - 1) as u32,
                                    y_local.max(0).min(vertex_count - 1) as u32,
                                )
                                .unwrap(),
                        ),
                    )
                })
                .collect::<HeightMap>();

            let mesh_3d = Mesh3d(meshes.add(build_mesh_data(heights, IVec2::splat(512))));

            commands.spawn((
                mesh_3d.clone(),
                Transform::from_scale(Vec3::new(size_meters.y, 1.0, size_meters.x)),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color_texture: Some(chunk.raster.clone()),
                    ..Default::default()
                })),
            ));
            commands.entity(entity).insert(ChunkLoaded);
        }
    });
}
