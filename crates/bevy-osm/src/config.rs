use bevy::prelude::*;

use crate::location::Location;

#[derive(Debug, PartialEq, Clone)]
pub enum RasterTileSource {
    OSMDefault,
    CesiumGoogle,
    Transport,
    Debug,
}

impl RasterTileSource {
    pub fn get_name(&self) -> String {
        match self {
            RasterTileSource::OSMDefault => "osm-default".into(),
            RasterTileSource::CesiumGoogle => "cesium-google".into(),
            RasterTileSource::Transport => "transport".into(),
            RasterTileSource::Debug => "debug".into(),
        }
    }
    pub fn get_extension(&self) -> String {
        match self {
            RasterTileSource::OSMDefault => "png".into(),
            RasterTileSource::CesiumGoogle => "jpg".into(),
            RasterTileSource::Transport => "png".into(),
            RasterTileSource::Debug => "".into(),
        }
    }
}

#[derive(Resource)]
pub struct OSMConfig {
    pub location: Location,
    pub ui_visible: bool,
    pub raster_tile_source: RasterTileSource,
}

impl Default for OSMConfig {
    fn default() -> Self {
        Self {
            location: Location::MonacoCenter,
            ui_visible: true,
            raster_tile_source: RasterTileSource::CesiumGoogle,
        }
    }
}
