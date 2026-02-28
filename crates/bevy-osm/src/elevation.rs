use std::{fs::File, io::Write, path::Path};

use bevy::log::debug;
use bevy_terrain::{
    mesh::{HeightMap, build_mesh_data, iterate_mesh_vertices},
    quadtree::ChunkLoaded,
};

use crate::chunk::Chunk;
use bevy::prelude::*;

const ELEVATION_BASE_URL: &str = "https://tiles.mapterhorn.com";
const RASTER_BASE_URL: &str = "https://tile.openstreetmap.org";
const HEIGHT_OFFSET: f32 = 130.0;
pub const TILE_VERTEX_COUNT: i32 = 64;
pub const TILE_PIXEL_COUNT: i32 = 512;
const DOWNSAMPLE_FACTOR: i32 = TILE_PIXEL_COUNT / TILE_VERTEX_COUNT;

pub fn cache_elevation_for_chunk(chunk: &Chunk) {
    chunk.ensure_cache_dirs_exist();

    let path_str = chunk.get_elevation_cache_path();
    let path = Path::new(&path_str);

    if !path.exists() {
        let (z, x, y) = (chunk.z, chunk.x, chunk.y);
        let url = format!("{ELEVATION_BASE_URL}/{z}/{x}/{y}.webp");
        let request = ehttp::Request::get(url.clone());
        debug!("Downloading elevation tile for {url}");

        ehttp::fetch(request, move |response| {
            let path = Path::new(&path_str);
            if let Ok(response) = response
                && response.ok
            {
                File::create(path)
                    .unwrap()
                    .write_all(&response.bytes)
                    .expect("Could not write to tile cache");
            } else {
                File::create(path)
                    .unwrap()
                    .write_all(include_bytes!("../../../assets/osm/empty-tile.webp"))
                    .expect("Could not write to tile cache");
            }
        });
    }
}

pub fn cache_raster_tile_for_chunk(chunk: &Chunk) {
    chunk.ensure_cache_dirs_exist();

    let path_str = chunk.get_osm_raster_cache_path();
    let path = Path::new(&path_str);

    if !path.exists() {
        let (z, x, y) = (chunk.z, chunk.x, chunk.y);
        let url = format!("{RASTER_BASE_URL}/{z}/{x}/{y}.png");
        let request = ehttp::Request::get(url.clone());
        debug!("Downloading raster tile for {url}");

        ehttp::fetch(request, move |response| {
            let path = Path::new(&path_str);
            if let Ok(response) = response
                && response.ok
            {
                File::create(path)
                    .unwrap()
                    .write_all(&response.bytes)
                    .expect("Could not write to tile cache");
            }
        });
    }
}

/// Source: https://github.com/tilezen/joerd/blob/master/docs/formats.md
pub fn elevation_color_to_height_meters(c: Color) -> f32 {
    let lin_color = c.to_srgba();
    (lin_color.red * 256.0 * 256.0 + lin_color.green * 256.0 + lin_color.blue)
        - 32768.0
        - HEIGHT_OFFSET
}

pub fn get_elevation_local(image: &Image, local_coords: IVec2) -> f32 {
    elevation_color_to_height_meters(
        image
            .get_color_at(
                // Clamp to border
                (DOWNSAMPLE_FACTOR * local_coords.y).clamp(0, TILE_PIXEL_COUNT - 1) as u32,
                (512 - DOWNSAMPLE_FACTOR * local_coords.x).clamp(0, TILE_PIXEL_COUNT - 1) as u32,
            )
            .unwrap(),
    )
}

pub fn spawn_elevation_meshes(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    heightmap: &Image,
    entity: Entity,
    chunk: Chunk,
) {
    let heights = iterate_mesh_vertices(IVec2::splat(TILE_VERTEX_COUNT), Rect::EMPTY)
        .map(|(x_local, y_local, ..)| {
            (
                (x_local, y_local),
                get_elevation_local(heightmap, IVec2::new(x_local, y_local)),
            )
        })
        .collect::<HeightMap>();

    let mesh = commands
        .spawn((
            Mesh3d(meshes.add(build_mesh_data(heights, IVec2::splat(TILE_VERTEX_COUNT)))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(chunk.raster),
                ..Default::default()
            })),
        ))
        .id();
    commands.entity(entity).insert(ChunkLoaded);
    commands.entity(entity).add_child(mesh);
}
