use bevy::prelude::*;
use bevy::render::view::Hdr;
use bevy_egui::EguiPlugin;
use bevy_fly_camera::system::{FlyCam, FlyCameraPlugin};
use bevy_volumetric_clouds::{CloudsPlugin, compute::CloudsConfig};
use bevy_where_was_i::{WhereWasI, WhereWasIPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Celestial".to_string(),
                    resolution: (1920, 1080).into(),
                    canvas: Some("#bevy".to_owned()),
                    prevent_default_event_handling: false,
                    fit_canvas_to_parent: true,
                    ..default()
                }),

                ..default()
            }),
            CloudsPlugin,
            FlyCameraPlugin,
            WhereWasIPlugin::default(),
        ))
        .add_plugins(EguiPlugin::default())
        .insert_resource(CloudsConfig::default())
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut std_materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Hdr,
        Projection::Perspective(PerspectiveProjection {
            near: 0.1,
            far: 1000.0,
            ..default()
        }),
        WhereWasI::from_name("clouds_camera"),
        FlyCam,
        Transform::from_translation(Vec3::new(4.5, 3.0, 1.5)).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Spawn ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(10000.0)))),
        MeshMaterial3d(std_materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
}
