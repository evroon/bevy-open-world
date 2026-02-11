use bevy::app::{App, Plugin, Startup};
use bevy_svg::SvgPlugin;

use crate::pfd::setup_pfd;

pub mod pfd;

pub struct FlightSimPlugin;

impl Plugin for FlightSimPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SvgPlugin).add_systems(Startup, setup_pfd);
    }
}
