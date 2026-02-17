use std::{collections::HashMap, fs::File, path::Path};

use bevy::{color::palettes::css::GREEN, log::info, math::Vec2};
use bevy_terrain::mesh::{build_mesh_data, iterate_mesh_vertices};
use serde::{Deserialize, Serialize};

use crate::location::Location;
use bevy::{camera::visibility::NoFrustumCulling, prelude::*};

#[derive(Serialize, Deserialize)]
pub struct TerrainData(pub Vec<(f32, f32, f32)>);

#[derive(Serialize, Deserialize)]
struct ElevationResponse {
    elevations: Vec<f32>,
}

const ELEVATION_BASE_URL: &str = "https://www.elevation-api.eu/v1/elevation";

pub fn get_elevation_for_coords(
    location: Location,
    coords: Vec<(Vec2, IVec2)>,
) -> HashMap<(i32, i32), f32> {
    location.ensure_cache_dir_exists();

    let path = location.get_elevation_path();
    let path = Path::new(&path);
    let mut result = HashMap::<(i32, i32), f32>::new();

    if !path.exists() {
        let mut data = TerrainData(Vec::new());
        info!("Downloading elevation data for {location}");

        coords.chunks(512).for_each(|chunk| {
            let chunk_joined = chunk
                .iter()
                .map(|(x, _)| format!("[{},{}]", x.x, x.y))
                .collect::<Vec<String>>()
                .join(",");

            let request = ehttp::Request::get(format!("{ELEVATION_BASE_URL}?pts=[{chunk_joined}]"));

            let response_raw = ehttp::fetch_blocking(&request);
            let response: std::result::Result<ElevationResponse, serde_json::Error> =
                serde_json::from_slice(&response_raw.unwrap().bytes);

            chunk
                .iter()
                .zip(response.expect("Invalid elevation API response").elevations)
                .for_each(|((global_coords, local_coords), elevation)| {
                    result.insert([local_coords.x, local_coords.y].into(), elevation);
                    data.0.push((global_coords.x, global_coords.y, elevation));
                });
        });
        serde_json::to_writer(File::create_new(path).unwrap(), &data).unwrap();

        info!("Finished downloading elevation data for {location}");
        return result;
    }

    let a: TerrainData = serde_json::from_reader(File::open(path).unwrap()).unwrap();
    coords
        .iter()
        .zip(a.0)
        .map(|((global_coord, local_coord), (lat, lon, elevation))| {
            assert_eq!(lat, global_coord.x);
            assert_eq!(lon, global_coord.y);
            ((local_coord.x, local_coord.y), elevation - 820.0)
        })
        .collect()
}

pub fn spawn_elevation_mesh(
    mut commands: Commands,
    mut meshes: ResMut<'_, Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let location = Location::MonacoCenter;
    let world_rect = location.get_area();
    let coords_to_world_scale = location.lat_lon_to_meters();
    let vertex_count = IVec2::splat(128);
    let material = MeshMaterial3d(materials.add(StandardMaterial {
        base_color: GREEN.into(),
        reflectance: 0.01,
        ..Default::default()
    }));

    let coords = iterate_mesh_vertices(vertex_count, world_rect)
        .map(|(x_local, y_local, lat, lon)| {
            (
                Vec2::new(lat as f32, lon as f32),
                IVec2::new(x_local, y_local),
            )
        })
        .collect();

    let mesh_3d = Mesh3d(meshes.add(build_mesh_data(
        get_elevation_for_coords(location, coords),
        vertex_count,
    )));

    commands.spawn((
        mesh_3d.clone(),
        Transform::from_scale(Vec3::new(
            world_rect.half_size().x * coords_to_world_scale.x,
            1.0,
            world_rect.half_size().y * coords_to_world_scale.y,
        )),
        material.clone(),
        NoFrustumCulling,
    ));
}
