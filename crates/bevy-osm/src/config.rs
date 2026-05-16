use bevy::prelude::*;

use crate::location::Location;

#[derive(Debug, PartialEq, Clone)]
pub enum RasterTileSource {
    OSMDefault,
    CesiumGoogleSatellite,
    CesiumGoogleRoadmaps,
    CesiumGoogleContour,
    Transport,
    Debug,
}

impl RasterTileSource {
    pub fn get_name(&self) -> String {
        match self {
            RasterTileSource::OSMDefault => "osm-default".into(),
            RasterTileSource::CesiumGoogleSatellite => "cesium-google-satellite".into(),
            RasterTileSource::CesiumGoogleRoadmaps => "cesium-google-maps".into(),
            RasterTileSource::CesiumGoogleContour => "cesium-google-contour".into(),
            RasterTileSource::Transport => "transport".into(),
            RasterTileSource::Debug => "debug".into(),
        }
    }
    pub fn get_extension(&self) -> String {
        match self {
            RasterTileSource::OSMDefault => "png".into(),
            RasterTileSource::CesiumGoogleSatellite => "jpg".into(),
            RasterTileSource::CesiumGoogleRoadmaps => "png".into(),
            RasterTileSource::CesiumGoogleContour => "jpg".into(),
            RasterTileSource::Transport => "png".into(),
            RasterTileSource::Debug => "".into(),
        }
    }
    pub fn get_cesium_asset_id(&self) -> String {
        match self {
            RasterTileSource::CesiumGoogleSatellite => "3830182".into(),
            RasterTileSource::CesiumGoogleRoadmaps => "3830184".into(),
            RasterTileSource::CesiumGoogleContour => "3830186".into(),
            _ => panic!("This RasterTileSource does not belong to Cesium"),
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
            raster_tile_source: RasterTileSource::CesiumGoogleSatellite,
        }
    }
}
