use bevy::app::{App, Plugin, Startup};
use bevy_prototype_lyon::plugin::ShapePlugin;

use crate::pfd::setup_pfd;

pub mod pfd;

pub struct FlightSimPlugin;

impl Plugin for FlightSimPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ShapePlugin).add_systems(Startup, setup_pfd);
    }
}
