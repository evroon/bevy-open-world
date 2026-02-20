use std::{fs::File, io::Write, path::Path};

use bevy::{color::palettes::css::GREEN, log::info, math::Vec2};
use bevy_terrain::mesh::iterate_mesh_vertices;

use crate::{chunk::Chunk, config::OSMConfig, location::Location};
use bevy::{camera::visibility::NoFrustumCulling, prelude::*};

const ELEVATION_BASE_URL: &str = "https://tiles.mapterhorn.com/";
const VERTICES_PER_TILE: i32 = 512;

pub fn get_elevation_for_chunk(location: Location, chunk: Chunk) -> String {
    location.ensure_cache_dir_exists();

    let path_str = chunk.get_cache_path();
    let path = Path::new(&path_str);

    if !path.exists() {
        info!("Downloading elevation data for {chunk:?}");

        let (z, x, y) = (chunk.z, chunk.x, chunk.y);

        let request = ehttp::Request::get(format!("{ELEVATION_BASE_URL}{z}/{x}/{y}"));

        let response_raw = ehttp::fetch_blocking(&request);
        File::create(path)
            .unwrap()
            .write_all(&response_raw.unwrap().bytes);

        info!("Finished downloading elevation data for {chunk:?}");
    }

    path_str
}

pub fn spawn_elevation_mesh(
    mut commands: Commands,
    mut meshes: ResMut<'_, Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<OSMConfig>,
) {
    let location = config.location.clone();
    let world_rect = location.get_area();
    let coords_to_world_scale = location.lat_lon_to_meters();
    let size_meters = world_rect.size() * coords_to_world_scale;

    info!("Terrain size in meters: {size_meters:?}");

    let material = MeshMaterial3d(materials.add(StandardMaterial {
        base_color: GREEN.into(),
        reflectance: 0.01,
        ..Default::default()
    }));

    let coords = iterate_mesh_vertices(IVec2::splat(VERTICES_PER_TILE), world_rect)
        .map(|(x_local, y_local, lat, lon)| {
            (
                Vec2::new(lat as f32, lon as f32),
                IVec2::new(x_local, y_local),
            )
        })
        .collect::<Vec<(Vec2, IVec2)>>();

    // let mesh_3d = Mesh3d(meshes.add(build_mesh_data(
    get_elevation_for_chunk(
        location,
        Chunk {
            z: 16,
            x: 34118,
            y: 23898,
        },
    );
    // vertex_count,
    // )));

    // commands.spawn((
    //     mesh_3d.clone(),
    //     Transform::from_scale(Vec3::new(size_meters.y, 1.0, size_meters.x)),
    //     material.clone(),
    //     NoFrustumCulling,
    // ));
}
