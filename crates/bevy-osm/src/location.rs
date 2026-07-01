use bevy::math::Vec2;
use strum_macros::Display;

#[derive(PartialEq, Eq, Hash, Display, Clone)]
pub enum Location {
    Amsterdam,
    London,
    Monaco,
    NewYork,
}

impl Location {
    pub fn get_world_center(&self) -> Vec2 {
        match self {
            Self::Amsterdam => Vec2::new(52.2798, 4.6026),
            Self::London => Vec2::new(51.509865, -0.118092),
            Self::Monaco => Vec2::new(43.71795, 7.38732),
            Self::NewYork => Vec2::new(40.70869, -73.99446),
        }
    }
    pub fn get_name(&self) -> String {
        match self {
            Self::Amsterdam => "Amsterdam".into(),
            Self::London => "London".into(),
            Self::Monaco => "Monaco".into(),
            Self::NewYork => "New York".into(),
        }
    }
}
