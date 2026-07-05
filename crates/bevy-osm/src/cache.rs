use std::{env, fs::File, path::Path};

use dotenvy::dotenv;
use serde::{Deserialize, Serialize};

use crate::{
    assets::mvt::MVTAsset,
    chunk::{ensure_cache_dir_exists, get_chunk_for_coord},
    config::{OSMConfig, RasterTileSource},
};
use bevy::prelude::*;

const ELEVATION_BASE_URL: &str = "https://tiles.mapterhorn.com";
const VECTOR_TILES_BASE_URL: &str = "https://tiles.openfreemap.org/planet/20260621_080001_pt";

fn get_token_cache_path(source: &RasterTileSource) -> String {
    let name = source.get_name();
    format!("assets/cache/{name}/token.json")
}

#[derive(Debug)]
pub enum DownloadUrlError {
    TokenFileInvalid,
    TokenFileAbsent,
}

fn get_raster_download_url(
    zxy: (u8, i32, i32),
    source: &RasterTileSource,
) -> Result<String, DownloadUrlError> {
    let (z, x, y) = zxy;

    match source {
        RasterTileSource::CesiumGoogleSatellite
        | RasterTileSource::CesiumGoogleRoadmaps
        | RasterTileSource::CesiumGoogleContour => {
            // panic!("session token would be in cache");
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

#[cfg(target_arch = "wasm32")]
pub fn load_raster_tile_for_chunk(
    zxy: (u8, i32, i32),
    config: &OSMConfig,
    asset_server: &Res<AssetServer>,
) -> Handle<Image> {
    asset_server.load("osm/empty-tile.webp")
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_raster_tile_for_chunk(
    zxy: (u8, i32, i32),
    config: &OSMConfig,
    asset_server: &Res<AssetServer>,
) -> Handle<Image> {
    use bevy::image::ImageFormatSetting;
    use bevy::image::ImageLoaderSettings;

    asset_server
        .load_builder()
        .with_settings(|settings: &mut ImageLoaderSettings| {
            settings.format = ImageFormatSetting::Format(ImageFormat::Jpeg)
        })
        .load(
            get_raster_download_url(zxy, &config.raster_tile_source)
                .expect("Could not get session after retrying"),
        )
}

#[cfg(target_arch = "wasm32")]
pub fn load_elevation_for_chunk(
    zxy: (u8, i32, i32),
    asset_server: &Res<AssetServer>,
) -> Handle<Image> {
    asset_server.load("osm/empty-tile.webp")
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_elevation_for_chunk(
    zxy: (u8, i32, i32),
    asset_server: &Res<AssetServer>,
) -> Handle<Image> {
    let (z, x, y) = zxy;
    let url = format!("{ELEVATION_BASE_URL}/{z}/{x}/{y}.webp");
    asset_server.load(url)
}

pub fn load_vector_tile_for_chunk(
    zxy: (u8, i32, i32),
    asset_server: &Res<AssetServer>,
) -> Handle<MVTAsset> {
    let (z, x, y) = zxy;
    let url = format!("{VECTOR_TILES_BASE_URL}/{z}/{x}/{y}.pbf");
    asset_server.load(url)
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

    let token_path = get_token_cache_path(source);

    ehttp::fetch(ehttp::Request::get(token_url), move |response| {
        if let Ok(success) = &response
            && success.ok
        {
            let json = success
                .json::<CesiumTokenResponse>()
                .expect("Received invalid JSON when fetching new Cesium session");
            let file = File::create(token_path).expect("Could not open token.json file");
            serde_json::to_writer_pretty(file, &json).expect("Could not write to token.json file");

            info!("saved new token.json");
        } else {
            panic!("Could not get new session from Cesium")
        }
    });
}

pub fn ensure_session_is_valid(source: &RasterTileSource) {
    let sampled_chunk = get_chunk_for_coord(0.0, 0.0, 0);
    let source_2 = source.clone();

    if let Ok(download_url) = get_raster_download_url(
        (sampled_chunk.z as u8, sampled_chunk.x, sampled_chunk.y),
        &source_2,
    ) {
        ehttp::fetch(ehttp::Request::get(download_url), move |response| {
            if response.is_err() || !response.unwrap().ok {
                get_new_session(&source_2);
            }
        });
    } else {
        get_new_session(source);
    }
}
