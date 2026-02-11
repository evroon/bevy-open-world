use crate::resources::{FillTessellator, StrokeTessellator};
use bevy::app::{App, Plugin};

use crate::render::svg2d;

/// Plugin that renders [`Svg`](crate::svg::Svg)s in 2D
pub struct SvgPlugin;

impl Plugin for SvgPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FillTessellator::default())
            .insert_resource(StrokeTessellator::default())
            .add_plugins(svg2d::RenderPlugin);
    }
}
