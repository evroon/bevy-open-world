use bevy::app::{App, Plugin, Startup};
use bevy_svg::SvgPlugin;

use crate::pfd::setup_pfd;

// pub mod asset_loader;
pub mod pfd;

pub struct FlightSimPlugin;

// /// A plugin that provides resources and a system to draw [`Svg`]s.
// pub struct SvgPlugin;

// impl Plugin for SvgPlugin {
//     #[inline]
//     fn build(&self, app: &mut App) {
//         app.init_asset::<Svg>()
//             .init_asset_loader::<SvgAssetLoader>();
//     }
// }

impl Plugin for FlightSimPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SvgPlugin).add_systems(Startup, setup_pfd);
    }
}
