use bevy::prelude::*;
use bevy_svg::prelude::*;

pub fn setup_pfd(mut commands: Commands, asset_server: Res<AssetServer>) {
    let svg = asset_server.load("textures/instruments/airbus-pfd.svg");
    commands.spawn((Camera2d, Msaa::Sample4));
    commands.spawn((Svg2d(svg), Origin::Center));
}
