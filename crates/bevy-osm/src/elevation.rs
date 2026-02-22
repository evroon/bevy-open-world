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
        let response = ehttp::fetch_blocking(&request).unwrap();

        if response.ok {
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
        let response = ehttp::fetch_blocking(&request).unwrap();

        if response.ok {
            File::create(path)
                .unwrap()
                .write_all(&response.bytes)
                .expect("Could not write to tile cache");
        }
    }
}

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
                local_coords.x.clamp(0, TILE_VERTEX_COUNT - 1) as u32,
                local_coords.y.clamp(0, TILE_VERTEX_COUNT - 1) as u32,
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
    config: &Res<OSMConfig>,
) {
    let world_rect = chunk.get_lat_lon_area();
    let area_meters = chunk.get_area_in_meters(config.location.get_world_center());
    let origin_meters = area_meters.center();
    let size_meters = area_meters.size();

    let heights = iterate_mesh_vertices(IVec2::splat(TILE_VERTEX_COUNT), world_rect)
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
            Transform::from_scale(Vec3::new(size_meters.x, 1.0, size_meters.y))
                .with_translation(Vec3::new(origin_meters.x, 0.0, origin_meters.y)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(chunk.raster),
                ..Default::default()
            })),
        ))
        .id();
    commands.entity(entity).insert(ChunkLoaded);
    commands.entity(entity).add_child(mesh);
}
