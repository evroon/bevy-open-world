use bevy::prelude::*;
use bevy_where_was_i::{WhereWasI, WhereWasIPlugin};

/// Example that saves the camera transform and restores it.
///
/// There's no content in this scene though, so the `3d_scene` example is more useful.
fn main() {
    App::new()
        .add_plugins(WhereWasIPlugin {
            directory: "./assets/saves/basic".into(),
        })
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands<'_, '_>) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        WhereWasI::camera(),
    ));
}
