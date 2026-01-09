use bevy::prelude::*;
pub mod system;

pub const EARTH_RADIUS: f32 = 20.0;

#[derive(Resource, Clone)]
pub struct EarthConfig {
    pub sun_direction: Quat,
    pub ui_visible: bool,
    pub emission_strength: f32,
    pub transition_fraction: f32,
    pub emission_threshold: f32,
}

impl Default for EarthConfig {
    fn default() -> Self {
        Self {
            sun_direction: Quat::from_rotation_y(0.0),
            ui_visible: true,
            emission_strength: 3000.0,
            emission_threshold: 0.01,
            transition_fraction: 0.1,
        }
    }
}
