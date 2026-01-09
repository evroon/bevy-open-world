use bevy::{color::palettes::css::WHITE, prelude::*, render::view::Hdr};
use bevy_osm::OSMPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_where_was_i::{WhereWasI, WhereWasIPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            OSMPlugin {},
            PanOrbitCameraPlugin,
            WhereWasIPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(AmbientLight {
        color: WHITE.into(),
        brightness: 500.0,
        ..default()
    });

    commands.spawn((
        Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        Hdr,
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            near: 0.01,
            far: 10.0,
            ..default()
        }),
        WhereWasI::from_name("osm_camera"),
        PanOrbitCamera::default(),
        Transform::from_translation(Vec3::new(4.5, 3.0, 1.5)).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            // base_color: Color::srgb(0.3, 0.3, 0.3),
            base_color: Color::srgb(0.2, 0.2, 0.2),
            reflectance: 0.1,
            perceptual_roughness: 0.7,
            clearcoat_perceptual_roughness: 0.7,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.52, 0.0).with_scale(Vec3::new(1000.0, 1.0, 1000.0)),
    ));
    // light
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
