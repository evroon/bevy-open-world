use std::{
    f32::consts::PI,
    f64::consts::PI as PI_64,
    fs::{self, File},
    io::Write,
    path::Path,
};

use bevy::math::ops::{atan, powf, sinh};
use bevy::prelude::*;
use osm::OSM;

// Assume at equator (110 km = 1 degree of longitude)
pub const LAT_LON_TO_METERS_CONVERSION: Vec2 = Vec2::splat(1.1e5);
const OVERPASS_BASE_URL: &str = "https://overpass-api.de/api/map";

pub fn ensure_cache_dir_exists(path: &Path) {
    fs::create_dir_all(Path::new(path).parent().unwrap())
        .expect("Could not create cache directory");
}
#[derive(Debug, PartialEq, Clone, Component)]
pub struct Chunk {
    pub x: i32,
    pub y: i32,
    pub z: i8,
    pub elevation: Handle<Image>,
    pub raster: Handle<Image>,
}
impl Chunk {
    pub fn get_osm_raster_cache_path(&self) -> String {
        let (z, x, y) = (self.z, self.x, self.y);
        format!("assets/cache/osm-raster/{z}/{x}/{y}.png")
    }
    pub fn get_osm_raster_cache_path_bevy(&self) -> String {
        let (z, x, y) = (self.z, self.x, self.y);
        format!("cache/osm-raster/{z}/{x}/{y}.png")
    }
    pub fn get_osm_cache_path(&self) -> String {
        let (z, x, y) = (self.z, self.x, self.y);
        format!("assets/cache/osm/{z}/{x}/{y}.osm")
    }
    pub fn get_elevation_cache_path(&self) -> String {
        let (z, x, y) = (self.z, self.x, self.y);
        format!("assets/cache/elevation/{z}/{x}/{y}.webp")
    }
    pub fn get_elevation_cache_path_bevy(&self) -> String {
        let (z, x, y) = (self.z, self.x, self.y);
        format!("cache/elevation/{z}/{x}/{y}.webp")
    }
    pub fn get_lat_lon_area(&self) -> Rect {
        let p0 = get_lat_lon(self.x as f32, self.y as f32, self.z);
        let p1 = get_lat_lon(1.0 + self.x as f32, 1.0 + self.y as f32, self.z);
        Rect::from_corners(
            Vec2::new(p0.0 as f32, p0.1 as f32),
            Vec2::new(p1.0 as f32, p1.1 as f32),
        )
    }
    pub fn get_area_in_meters(&self, lat_lon_origin: Vec2) -> Rect {
        let origin = lat_lon_to_world(self.get_lat_lon_area().center(), lat_lon_origin);
        Rect::from_center_size(
            Vec2::new(origin.0 as f32, origin.1 as f32),
            self.get_size_in_meters(),
        )
    }
    pub fn get_size_in_meters(&self) -> Vec2 {
        self.get_lat_lon_area().size().yx() * LAT_LON_TO_METERS_CONVERSION
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
        raster: Handle::default(),
    }
}

pub fn get_lat_lon(x: f32, y: f32, zoom: i8) -> (f64, f64) {
    (
        (atan(sinh(PI - y / powf(2.0, zoom as f32) * 2.0 * PI)) * 180.0 / PI) as f64,
        (x / powf(2.0, zoom as f32) * 360.0 - 180.0) as f64,
    )
}

/// The area of the location in lat, lon coordinates (degrees)
pub fn lat_lon_to_world(lat_lon: Vec2, lat_lon_origin: Vec2) -> (f64, f64) {
    (
        // 1. We need to switch (lat, lon) to (lon, lat)
        // 2. We need to invert the lat coordinates on z-axis because Bevy's coordinate
        //    system has the Z-axis pointed downwards (instead of upwards) when X-axis
        //    points to the right.
        (lat_lon.y as f64 - lat_lon_origin.y as f64) * LAT_LON_TO_METERS_CONVERSION.y as f64,
        -(lat_lon.x as f64 - lat_lon_origin.x as f64) * LAT_LON_TO_METERS_CONVERSION.x as f64,
    )
}

/// The area of the location in lat, lon coordinates (degrees)
pub fn world_to_lat_lon(pos: Vec3, lat_lon_origin: Vec2) -> (f32, f32) {
    (
        -pos.z / LAT_LON_TO_METERS_CONVERSION.x + lat_lon_origin.x,
        pos.x / LAT_LON_TO_METERS_CONVERSION.y + lat_lon_origin.y,
    )
}

pub fn get_osm_for_chunk(chunk: Chunk) -> OSM {
    let osm_path = chunk.get_osm_cache_path();
    let path = Path::new(&osm_path);
    ensure_cache_dir_exists(path);

    if !path.exists() {
        info!("Downloading OSM data for {chunk:?}");
        let area = chunk.get_lat_lon_area();
        let request = ehttp::Request::get(format!(
            "{OVERPASS_BASE_URL}?bbox={},{},{},{}",
            area.min.y, area.min.x, area.max.y, area.max.x
        ));
        let response = ehttp::fetch_blocking(&request);

        if let Ok(response) = response {
            let response_text = response.text().unwrap();
            if response_text.contains("The server is probably too busy to handle your request")
                || response_text.contains("rate_limited")
            {
                error!("failed to query Overpass API");
            }

            File::create_new(path)
                .unwrap()
                .write_all(&response.bytes)
                .expect("failed to write Overpass data");
        } else {
            error!("failed to query Overpass API");
        }

        info!("Finished downloading OSM data for {chunk:?}");
    }

    let file = File::open(path).unwrap();
    osm::OSM::parse(file).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_float_eq(v1: f64, v2: f64) {
        assert!((v1 - v2).abs() < 1e-5, "{v1} - {v2} >= 1e-5");
    }

    fn get_chunk_with_coordinates() -> (Chunk, (f64, f64)) {
        (
            Chunk {
                x: 140178,
                y: 97411,
                z: 18,
                elevation: Handle::default(),
                raster: Handle::default(),
            },
            (41.89921, 12.505188),
        )
    }

    #[test]
    fn test_chunk_to_lat_lon_conversion() {
        let (chunk, (lat_expected, lon_expected)) = get_chunk_with_coordinates();
        let (lat, lon) = get_lat_lon(chunk.x as f32, chunk.y as f32, chunk.z);
        assert_float_eq(lat, lat_expected);
        assert_float_eq(lon, lon_expected);
    }

    #[test]
    fn test_lat_lon_to_chunk_conversion() {
        let (chunk_expected, (lat, lon)) = get_chunk_with_coordinates();
        let chunk = get_chunk_for_coord(lat, lon, chunk_expected.z);
        assert_eq!(get_chunk_for_coord(lat, lon, chunk.z), chunk);
    }

    #[test]
    fn test_get_lat_lon() {
        assert_eq!(get_lat_lon(1.0, 1.0, 1), (0.0, 0.0));

        let northwest = get_lat_lon(0.0, 0.0, 0);
        assert_float_eq(northwest.0, 85.051125);
        assert_float_eq(northwest.1, -180.0);
    }
}
