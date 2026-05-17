use std::{env, fs::File, io::Write, path::Path};

use bevy::log::debug;
use dotenvy::dotenv;
use ehttp::Response;
use serde::{Deserialize, Serialize};

use crate::{
    chunk::{Chunk, ensure_cache_dir_exists, get_chunk_for_coord},
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
pub fn get_token_cache_path(source: &RasterTileSource) -> String {
    let name = source.get_name();
    format!("assets/cache/{name}/token.json")
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

#[derive(Debug)]
pub enum DownloadUrlError {
    TokenFileInvalid,
    TokenFileAbsent,
}

fn get_download_url(chunk: &Chunk, source: &RasterTileSource) -> Result<String, DownloadUrlError> {
    let (z, x, y) = (chunk.z, chunk.x, chunk.y);

    match source {
        RasterTileSource::CesiumGoogleSatellite
        | RasterTileSource::CesiumGoogleRoadmaps
        | RasterTileSource::CesiumGoogleContour => {
            dotenv().expect("Could not read .env");

            // TODO: don't read file for every tile fetch
            let file = File::open(get_token_cache_path(source))
                .or(Err(DownloadUrlError::TokenFileAbsent))?;
            let secrets: CesiumTokenResponse =
                serde_json::from_reader(file).or(Err(DownloadUrlError::TokenFileInvalid))?;

            let asset_id = source.get_cesium_asset_id();
            let key = secrets.options.key;
            let session = secrets.options.session;

            Ok(format!(
                "https://assets.ion.cesium.com/proxy/{asset_id}/v1/2dtiles/{z}/{x}/{y}?session={session}&key={key}"
            ))
        }
        RasterTileSource::Transport => Ok(format!(
            "https://tileserver.memomaps.de/tilegen/{z}/{x}/{y}.png"
        )),
        RasterTileSource::OSMDefault | RasterTileSource::Debug => {
            Ok(format!("https://tile.openstreetmap.org/{z}/{x}/{y}.png"))
        }
    }
}

pub fn cache_raster_tile_for_chunk(chunk: &Chunk, config: &OSMConfig) {
    let path_str = get_osm_raster_cache_path(chunk, config);

    let download_url = get_download_url(chunk, &config.raster_tile_source);
    let error_handler = |_, res: Result<Response, String>| {
        match res {
            Ok(res) => {
                error!(
                    "Raster tile download error [{:?}]: {:?}",
                    res.status,
                    res.text()
                );
            }
            Err(err) => {
                error!("Raster tile unknown download error: {:?}", err);
            }
        };
    };

    if let Ok(download_url) = download_url {
        cache_tile_for_chunk(path_str, download_url, error_handler);
    } else {
        // Try again
        get_new_session(&config.raster_tile_source);

        let download_url = get_download_url(chunk, &config.raster_tile_source)
            .expect("Could not get session after retrying");
        cache_tile_for_chunk(path_str, download_url, error_handler);
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[expect(non_snake_case)]
struct CesiumTokenOptions {
    session: String,
    key: String,
    imageFormat: String,
    tileWidth: u32,
    tileHeight: u32,
    url: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[expect(non_snake_case)]
struct CesiumTokenResponse {
    externalType: String,
    r#type: String,
    options: CesiumTokenOptions,
}

pub fn get_new_session(source: &RasterTileSource) {
    let asset_id = source.get_cesium_asset_id();

    let binding = get_token_cache_path(source);
    let path = Path::new(&binding);
    ensure_cache_dir_exists(path);

    let access_token = env::var("CESIUM_ACCESS_TOKEN").expect("Could not read CESIUM_ACCESS_TOKEN");
    let token_url =
        format!("https://api.cesium.com/v1/assets/{asset_id}/endpoint?access_token={access_token}");

    let response = ehttp::fetch_blocking(&ehttp::Request::get(token_url));

    if let Ok(success) = &response
        && success.ok
    {
        let json = success
            .json::<CesiumTokenResponse>()
            .expect("Received invalid JSON when fetching new Cesium session");
        let file =
            File::create(get_token_cache_path(source)).expect("Could not open token.json file");
        serde_json::to_writer_pretty(file, &json).expect("Could not write to token.json file");

        info!("saved new token.json");
    } else {
        panic!("Could not get new session from Cesium")
    }
}

pub fn ensure_session_is_valid(source: &RasterTileSource) {
    let sampled_chunk = get_chunk_for_coord(0.0, 0.0, 0);

    if let Ok(download_url) = get_download_url(&sampled_chunk, source) {
        let response = ehttp::fetch_blocking(&ehttp::Request::get(download_url));

        if let Ok(success) = &response
            && success.ok
        {
            // Session is still valid
            return;
        }
    }

    get_new_session(source);
}
