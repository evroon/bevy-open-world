use std::{
    f32::consts::PI,
    f64::consts::PI as PI_64,
    fs::{self, File},
    io::Write,
    path::Path,
};

use bevy::{
    asset::Handle,
    ecs::component::Component,
    image::Image,
    log::info,
    math::{
        Rect, Vec2,
        ops::{atan, powf, sinh},
    },
};
use osm::OSM;

const OVERPASS_BASE_URL: &str = "https://overpass-api.de/api/map";

#[derive(Component)]
pub struct ChunkLoaded;

#[derive(Debug, PartialEq, Clone, Component)]
pub struct Chunk {
    pub x: i32,
    pub y: i32,
    pub z: i8,
    pub elevation: Handle<Image>,
    // pub mesh: Option<Handle<Mesh>>,
}
impl Chunk {
    pub fn ensure_cache_dirs_exist(&self) {
        for p in [
            &self.get_elevation_cache_path(),
            &self.get_osm_cache_path(),
            &self.get_osm_raster_cache_path(),
        ] {
            fs::create_dir_all(Path::new(p).parent().unwrap())
                .expect("Could not create cache directory");
        }
    }
    pub fn get_osm_raster_cache_path(&self) -> String {
        let (z, x, y) = (self.z, self.x, self.y);
        format!("assets/osm-raster/v1/{z}/{x}/{y}.osm")
    }
    pub fn get_osm_cache_path(&self) -> String {
        let (z, x, y) = (self.z, self.x, self.y);
        format!("assets/osm/v1/{z}/{x}/{y}.osm")
    }
    pub fn get_elevation_cache_path(&self) -> String {
        let (z, x, y) = (self.z, self.x, self.y);
        format!("assets/elevation/v1/{z}/{x}/{y}.webp")
    }
    pub fn get_elevation_cache_path_bevy(&self) -> String {
        let (z, x, y) = (self.z, self.x, self.y);
        format!("elevation/v1/{z}/{x}/{y}.webp")
    }
    pub fn get_lat_lon_area(&self) -> Rect {
        let p0 = get_lat_lon(self.x as f32, self.y as f32, self.z);
        let p1 = get_lat_lon(1.0 + self.x as f32, 1.0 + self.y as f32, self.z);
        Rect::from_corners(
            Vec2::new(p0.0 as f32, p0.1 as f32),
            Vec2::new(p1.0 as f32, p1.1 as f32),
        )
    }
    pub fn lat_lon_to_meters(&self) -> Vec2 {
        // Assume at equator
        Vec2::splat(1.1e5)
    }
    #[inline]
    /// The area of the location in lat, lon coordinates (degrees)
    pub fn lat_lon_to_world(&self, lat: f64, lon: f64) -> (f64, f64) {
        // 1. We need to switch (lat, lon) to (lon, lat)
        // 2. We need to invert the lat coordinates on z-axis because Bevy's coordinate
        //    system has the Z-axis pointed downwards (instead of upwards) when X-axis
        //    points to the right.
        (
            (lon - self.get_lat_lon_area().center().y as f64) * self.lat_lon_to_meters().y as f64,
            -(lat - self.get_lat_lon_area().center().x as f64) * self.lat_lon_to_meters().x as f64,
        )
    }
}

pub fn get_chunk_for_coord(lat_deg: f64, lon_deg: f64, zoom: i8) -> Chunk {
    let n = (1 << zoom) as f64;

    let x_tile = (n * (lon_deg + 180.0) / 360.0) as i32;

    let lat_rad = lat_deg.to_radians();
    let y_tile = (n * (1.0 - (lat_rad.tan() + (1.0 / lat_rad.cos())).ln() / PI_64) / 2.0) as i32;

    Chunk {
        x: x_tile,
        y: y_tile,
        z: zoom,
        elevation: Handle::default(),
    }
}

pub fn get_lat_lon(x: f32, y: f32, zoom: i8) -> (f64, f64) {
    (
        (atan(sinh(PI - y / powf(2.0, zoom as f32) * 2.0 * PI)) * 180.0 / PI) as f64,
        (x / powf(2.0, zoom as f32) * 360.0 - 180.0) as f64,
    )
}

pub fn get_osm_for_chunk(chunk: Chunk) -> OSM {
    chunk.ensure_cache_dirs_exist();

    let osm_path = chunk.get_osm_cache_path();
    let path = Path::new(&osm_path);

    if !path.exists() {
        info!("Downloading OSM data for {chunk:?}");
        let area = chunk.get_lat_lon_area();
        let request = ehttp::Request::get(format!(
            "{OVERPASS_BASE_URL}?bbox={},{},{},{}",
            area.min.y, area.min.x, area.max.y, area.max.x
        ));
        let response = ehttp::fetch_blocking(&request);

        File::create_new(path)
            .unwrap()
            .write_all(&response.expect("failed to query Overpass API").bytes)
            .expect("failed to write Overpass data");

        info!("Finished downloading OSM data for {chunk:?}");
    }

    let file = File::open(path).unwrap();
    osm::OSM::parse(file).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_chunk_with_coordinates() -> (Chunk, (f64, f64)) {
        (
            Chunk {
                x: 140178,
                y: 97411,
                z: 18,
                elevation: Handle::default(),
            },
            (41.89921, 12.505188),
        )
    }

    #[test]
    fn test_chunk_to_lat_lon_conversion() {
        let (chunk, coords) = get_chunk_with_coordinates();
        let (lat, lon) = get_lat_lon(chunk.x as f32, chunk.y as f32, chunk.z);
        assert_eq!((lat, lon), coords);
    }

    #[test]
    fn test_lat_lon_to_chunk_conversion() {
        let (chunk, (lat, lon)) = get_chunk_with_coordinates();
        assert_eq!(get_chunk_for_coord(lat, lon, chunk.z), chunk);
    }
}
