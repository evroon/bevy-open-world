use std::{env, fs::File, io::Write, path::Path};

use bevy::log::debug;
use dotenvy::dotenv;
use ehttp::Response;

use crate::{
    chunk::{Chunk, ensure_cache_dir_exists},
    config::{OSMConfig, RasterTileSource},
};
use bevy::prelude::*;

const ELEVATION_BASE_URL: &str = "https://tiles.mapterhorn.com";

pub fn get_osm_raster_cache_path(chunk: &Chunk, config: &OSMConfig) -> String {
    let (z, x, y) = (chunk.z, chunk.x, chunk.y);
    let name = config.raster_tile_source.get_name();
    let extension = config.raster_tile_source.get_extension();
    format!("assets/cache/{name}/{z}/{x}/{y}.{extension}")
}
pub fn get_osm_raster_cache_path_bevy(chunk: &Chunk, config: &OSMConfig) -> String {
    let (z, x, y) = (chunk.z, chunk.x, chunk.y);
    let name = config.raster_tile_source.get_name();
    let extension = config.raster_tile_source.get_extension();
    format!("cache/{name}/{z}/{x}/{y}.{extension}")
}
pub fn get_osm_cache_path(chunk: &Chunk) -> String {
    let (z, x, y) = (chunk.z, chunk.x, chunk.y);
    format!("assets/cache/osm/{z}/{x}/{y}.osm")
}
pub fn get_elevation_cache_path(chunk: &Chunk) -> String {
    let (z, x, y) = (chunk.z, chunk.x, chunk.y);
    format!("assets/cache/elevation/{z}/{x}/{y}.webp")
}
pub fn get_elevation_cache_path_bevy(chunk: &Chunk) -> String {
    let (z, x, y) = (chunk.z, chunk.x, chunk.y);
    format!("cache/elevation/{z}/{x}/{y}.webp")
}

fn cache_tile_for_chunk(
    path_str: String,
    url: String,
    error_handler: impl 'static + Send + FnOnce(String, Result<Response, String>),
) {
    let path = Path::new(&path_str);
    ensure_cache_dir_exists(path);

    if !path.exists() {
        let request = ehttp::Request::get(url.clone());
        debug!("Downloading elevation tile for {url}");

        ehttp::fetch(request, move |response| {
            let path = Path::new(&path_str);
            if let Ok(success) = &response
                && success.ok
            {
                File::create(path)
                    .unwrap()
                    .write_all(&success.bytes)
                    .expect("Could not write to tile cache");
            } else {
                error_handler(path_str, response);
            }
        });
    }
}

pub fn cache_elevation_for_chunk(chunk: &Chunk) {
    let (z, x, y) = (chunk.z, chunk.x, chunk.y);
    let url = format!("{ELEVATION_BASE_URL}/{z}/{x}/{y}.webp");

    let on_error = move |path_str, _| {
        let path = Path::new(&path_str);

        File::create(path)
            .unwrap()
            .write_all(include_bytes!("../../../assets/osm/empty-tile.webp"))
            .expect("Could not write to tile cache");
    };

    let path_str = get_elevation_cache_path(chunk);
    cache_tile_for_chunk(path_str, url, on_error);
}

pub fn cache_raster_tile_for_chunk(chunk: &Chunk, config: &OSMConfig) {
    let (z, x, y) = (chunk.z, chunk.x, chunk.y);

    let url = match config.raster_tile_source {
        RasterTileSource::CesiumGoogle => {
            dotenv().expect("Could not read .env");
            let asset_id = env::var("CESIUM_ASSET_ID").expect("Could not read CESIUM_ASSET_ID");
            let key = env::var("CESIUM_KEY").expect("Could not read CESIUM_KEY");
            let session = env::var("CESIUM_SESSION").expect("Could not read CESIUM_SESSION");
            format!(
                "https://assets.ion.cesium.com/proxy/{asset_id}/v1/2dtiles/{z}/{x}/{y}?session={session}&key={key}"
            )
        }
        RasterTileSource::Transport => {
            format!("https://tileserver.memomaps.de/tilegen/{z}/{x}/{y}.png")
        }
        _ => format!("https://tile.openstreetmap.org/{z}/{x}/{y}.png"),
    };

    let path_str = get_osm_raster_cache_path(chunk, config);
    cache_tile_for_chunk(path_str, url, |_, res| error!("{:?}", res));
}
