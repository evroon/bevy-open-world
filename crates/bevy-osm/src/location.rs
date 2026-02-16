use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use bevy::{
    log::info,
    math::{Rect, Vec2},
};
use osm::OSM;
use strum_macros::Display;

const OVERPASS_BASE_URL: &str = "https://overpass-api.de/api/map";

#[derive(PartialEq, Eq, Hash, Display, Clone)]
pub enum Location {
    MonacoCenter,
    MonacoFull,
}

impl Location {
    fn get_path(&self) -> &str {
        match self {
            Self::MonacoCenter => "assets/osm/monaco-center/",
            Self::MonacoFull => "assets/osm/monaco-full/",
        }
    }
    pub fn ensure_cache_dir_exists(&self) {
        fs::create_dir_all(self.get_path()).expect("Could not create directory");
    }
    pub fn get_osm_path(&self) -> String {
        self.get_path().to_owned() + "map.osm"
    }
    pub fn get_elevation_path(&self) -> String {
        self.get_path().to_owned() + "elevation.json"
    }
    pub fn lat_lon_to_meters(&self) -> Vec2 {
        // Assume at equator
        Vec2::splat(1.1e5)
    }
    /// The area of the location in lat, lon coordinates
    pub fn get_area(&self) -> Rect {
        match self {
            Self::MonacoCenter => {
                Rect::from_corners(Vec2::new(43.72264, 7.40848), Vec2::new(43.73864, 7.43320))
            }
            Self::MonacoFull => {
                Rect::from_corners(Vec2::new(43.71795, 7.38732), Vec2::new(43.75758, 7.45083))
            }
        }
    }
}

pub fn get_osm_for_location(location: Location) -> OSM {
    location.ensure_cache_dir_exists();

    let osm_path = location.get_osm_path();
    let path = Path::new(&osm_path);

    if !path.exists() {
        info!("Downloading OSM data for {location}");
        let area = location.get_area();
        let request = ehttp::Request::get(format!(
            "{OVERPASS_BASE_URL}?bbox={},{},{},{}",
            area.min.y, area.min.x, area.max.y, area.max.x
        ));
        let response = ehttp::fetch_blocking(&request);

        File::create_new(path)
            .unwrap()
            .write_all(&response.expect("failed to query Overpass API").bytes)
            .expect("failed to write Overpass data");

        info!("Finished downloading OSM data for {location}");
    }

    let file = File::open(path).unwrap();
    osm::OSM::parse(file).unwrap()
}
