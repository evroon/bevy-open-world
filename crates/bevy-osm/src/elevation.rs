use std::{fs::File, io::Write, path::Path};

use bevy::log::info;
use bevy_terrain::mesh::{HeightMap, build_mesh_data, iterate_mesh_vertices};

use crate::chunk::{Chunk, ChunkLoaded};
use bevy::prelude::*;

const ELEVATION_BASE_URL: &str = "https://tiles.mapterhorn.com";
const RASTER_BASE_URL: &str = "https://tile.openstreetmap.org";
pub const TILE_VERTEX_COUNT: i32 = 512;
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

pub fn elevation_color_to_height_meters(c: Color) -> f32 {
    let lin_color = c.to_srgba();
    (lin_color.red * 256.0 * 256.0 + lin_color.green * 256.0 + lin_color.blue)
        - 32768.0
        - HEIGHT_OFFSET
}

pub fn get_elevation_local(image: &Image, x_local: i32, y_local: i32) -> f32 {
    elevation_color_to_height_meters(
        image
            .get_color_at(
                // Clamp to border
                x_local.clamp(0, TILE_VERTEX_COUNT - 1) as u32,
                y_local.clamp(0, TILE_VERTEX_COUNT - 1) as u32,
            )
            .unwrap(),
    )
}

pub fn spawn_elevation_meshes(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    image: &Image,
    entity: Entity,
    chunk: Chunk,
) {
    let world_rect = chunk.get_lat_lon_area();
    let size_meters = chunk.get_size_in_meters();
    let origin_meters = chunk.get_lat_lon_area().center();

    info!("Terrain size in meters: {size_meters:?}");
    info!("Terrain size in lat, lon: {world_rect:?}");

    let heights = iterate_mesh_vertices(IVec2::splat(TILE_VERTEX_COUNT), world_rect)
        .map(|(x_local, y_local, ..)| {
            (
                (x_local, y_local),
                get_elevation_local(image, x_local, y_local),
            )
        })
        .collect::<HeightMap>();

    commands.spawn((
        Mesh3d(meshes.add(build_mesh_data(heights, IVec2::splat(TILE_VERTEX_COUNT)))),
        Transform::from_scale(Vec3::new(size_meters.y, 1.0, size_meters.x))
            .with_translation(Vec3::new(origin_meters.y, 0.0, origin_meters.x)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(chunk.raster),
            ..Default::default()
        })),
    ));
    commands.entity(entity).insert(ChunkLoaded);
}
