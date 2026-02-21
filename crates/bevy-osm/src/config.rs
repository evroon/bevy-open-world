use bevy::prelude::*;

use crate::location::Location;
#[derive(Resource)]
pub struct OSMConfig {
    pub location: Location,
}

impl Default for OSMConfig {
    fn default() -> Self {
        Self {
            location: Location::MonacoCenter,
        }
    }
}
