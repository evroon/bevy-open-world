use bevy::{color::palettes::css::*, prelude::*};
use bevy_prototype_lyon::prelude::*;

pub fn setup_pfd(mut commands: Commands, asset_server: Res<AssetServer>) {
    let rect = shapes::Rectangle {
        extents: Vec2::splat(175.0),
        origin: RectangleOrigin::Center,
        radii: Some(BorderRadii::single(25.0)),
    };

    commands.spawn((Camera2d, Msaa::Sample4));

    commands.spawn(Sprite::from_image(
        asset_server.load("textures/instruments/airbus-pfd.png"),
    ));
    commands.spawn(ShapeBuilder::with(&rect).fill(RED).build());
}
