use bevy::prelude::*;

use crate::location::Location;

#[derive(Debug, PartialEq, Clone)]
pub enum RasterTileSource {
    OSMDefault,
    CesiumGoogle,
    Debug,
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
            raster_tile_source: RasterTileSource::OSMDefault,
        }
    }
}
