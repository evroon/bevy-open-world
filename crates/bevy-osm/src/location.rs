use bevy::math::Vec2;
use strum_macros::Display;

#[derive(PartialEq, Eq, Hash, Display, Clone)]
pub enum Location {
    Amsterdam,
    Monaco,
    NewYork,
}

impl Location {
    pub fn get_world_center(&self) -> Vec2 {
        match self {
            Self::Monaco => Vec2::new(43.71795, 7.38732),
            Self::Amsterdam => Vec2::new(52.2798, 4.6026),
            Self::NewYork => Vec2::new(40.70869, -73.99446),
        }
    }
    pub fn get_name(&self) -> String {
        match self {
            Self::Monaco => "Monaco".into(),
            Self::Amsterdam => "Amsterdam".into(),
            Self::NewYork => "New York".into(),
        }
    }
}
