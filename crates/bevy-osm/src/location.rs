use bevy::math::Vec2;
use strum_macros::Display;

#[derive(PartialEq, Eq, Hash, Display, Clone)]
pub enum Location {
    MonacoCenter,
    MonacoFull,
}

impl Location {
    pub fn get_world_center(&self) -> Vec2 {
        match self {
            Self::MonacoCenter => Vec2::new(43.72264, 7.40848),
            Self::MonacoFull => Vec2::new(43.71795, 7.38732),
        }
    }
}
