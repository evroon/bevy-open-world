//! Load and disply simple SVG files in Bevy.
//!
//! This crate provides a Bevy `Plugin` to easily load and display a simple SVG file.
//!
//! ## Usage
//! Simply add the crate in your `Cargo.toml` and add the plugin to your app
//!
//! ```rust
//!     App::new()
//!         .add_plugin(bevy_svg::prelude::SvgPlugin)
//!         .run();
//! ```

#![allow(clippy::type_complexity)]

mod loader;
mod origin;
mod plugin;
mod render;
mod resources;
mod svg;
mod util;

/// Import this module as `use bevy_svg::prelude::*` to get convenient imports.
pub mod prelude {
    pub use crate::loader::SvgAssetLoader;
    pub use crate::origin::Origin;
    pub use crate::plugin::SvgSet;
    pub use crate::render::Svg2d;
    pub use crate::svg::Svg;
    pub use lyon_tessellation::{
        FillOptions, FillRule, LineCap, LineJoin, Orientation, StrokeOptions,
    };
}

use crate::plugin::SvgRenderPlugin;
use crate::{loader::SvgAssetLoader, svg::Svg};
use bevy::{
    app::{App, Plugin},
    asset::AssetApp,
};

/// A plugin that provides resources and a system to draw [`Svg`]s.
pub struct SvgPlugin;

impl Plugin for SvgPlugin {
    #[inline]
    fn build(&self, app: &mut App) {
        app.init_asset::<Svg>()
            .init_asset_loader::<SvgAssetLoader>();
        app.add_plugins(SvgRenderPlugin);
    }
}

/// A locally defined [`std::convert::Into`] surrogate to overcome orphan rules.
pub trait Convert<T>: Sized {
    /// Converts the value to `T`.
    fn convert(self) -> T;
}
