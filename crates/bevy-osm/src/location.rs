use bevy::math::Vec2;
use strum_macros::Display;

use crate::chunk::{Chunk, get_chunk_for_coord};

#[derive(PartialEq, Eq, Hash, Display, Clone)]
pub enum Location {
    MonacoCenter,
    MonacoFull,
}

impl Location {
    pub fn get_chunk(&self) -> Chunk {
        let center = match self {
            Self::MonacoCenter => Vec2::new(43.72264, 7.40848),
            Self::MonacoFull => Vec2::new(43.71795, 7.38732),
        };
        get_chunk_for_coord(center.x as f64, center.y as f64, 13)
    }
    pub fn get_world_center(&self) -> Vec2 {
        self.get_chunk().get_lat_lon_area().center()
    }
}
