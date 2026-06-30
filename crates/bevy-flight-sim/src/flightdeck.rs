use bevy::prelude::*;

pub fn spawn_flightdeck(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        WorldAssetRoot(
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/a320/A320.glb")),
        ),
        Transform::from_translation(Vec3::Y * 2000.0),
    ));
}
