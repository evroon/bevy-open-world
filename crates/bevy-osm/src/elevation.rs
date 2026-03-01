use std::{f32::consts::PI, fs::File, io::Write, path::Path};

use bevy::{
    color::palettes::css::{BLUE, FUCHSIA, GREEN, INDIGO, RED, TEAL, WHITE},
    log::info,
    math::Affine2,
};
use bevy_terrain::{
    mesh::{HeightMap, build_mesh_data, iterate_mesh_vertices},
    quadtree::ChunkLoaded,
};

use crate::{chunk::Chunk, config::OSMConfig};
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
        info!("Downloading elevation tile for {url}");

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
        info!("Downloading raster tile for {url}");

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
                (DOWNSAMPLE_FACTOR * local_coords.x).clamp(0, TILE_PIXEL_COUNT - 1) as u32,
                (DOWNSAMPLE_FACTOR * local_coords.y).clamp(0, TILE_PIXEL_COUNT - 1) as u32,
            )
            .unwrap(),
    )
}

fn _debug_material(
    materials: &mut ResMut<Assets<StandardMaterial>>,
    chunk: &Chunk,
) -> MeshMaterial3d<StandardMaterial> {
    MeshMaterial3d(materials.add(StandardMaterial {
        base_color: match chunk.z {
            11 => TEAL.into(),
            12 => FUCHSIA.into(),
            13 => RED.into(),
            14 => GREEN.into(),
            15 => BLUE.into(),
            16 => INDIGO.into(),
            _ => WHITE.into(),
        },
        ..Default::default()
    }))
}

pub fn spawn_elevation_meshes(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    heightmap: &Image,
    entity: Entity,
    chunk: Chunk,
    _config: &Res<OSMConfig>,
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
            // Transform::from_scale(Vec3::new(size_meters.x, 1.0, size_meters.y))
            //     .with_translation(Vec3::new(origin_meters.x, 0.0, origin_meters.y)),
            Transform::IDENTITY,
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(chunk.raster),
                uv_transform: Affine2::from_angle_translation(PI * 0.5, Vec2::new(1.0, 0.0)),
                ..Default::default()
            })),
        ))
        .id();
    commands.entity(entity).insert(ChunkLoaded);
    commands.entity(entity).add_child(mesh);
}
