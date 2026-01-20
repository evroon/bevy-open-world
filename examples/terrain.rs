use bevy::DefaultPlugins;
use bevy::asset::AssetMetaCheck;
use bevy::color::palettes::css::WHITE;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::pbr::ExtendedMaterial;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::{RenderCreation, WgpuFeatures, WgpuSettings};
use bevy::render::view::Hdr;
use bevy_egui::EguiPlugin;
use bevy_skybox::system::{init_skybox, update_skybox};
use bevy_terrain::build_terrain_tile;
use bevy_terrain::material::PlanetMaterial;
use bevy_terrain::mesh::build_mesh_cache;
use bevy_terrain::system::update_terrain_quadtree;
use bevy_volumetric_clouds::fly_camera::{FlyCam, FlyCameraPlugin};
use bevy_where_was_i::{WhereWasI, WhereWasIPlugin};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::linear_rgb(0.4, 0.4, 0.4)))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bevy Open World".to_string(),
                        resolution: (1920, 1040).into(),
                        canvas: Some("#bevy".to_owned()),
                        prevent_default_event_handling: false,
                        fit_canvas_to_parent: true,
                        ..default()
                    }),

                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        // WARN this is a native only feature. It will not work with webgl or webgpu
                        features: WgpuFeatures::POLYGON_MODE_LINE,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((
            EguiPlugin::default(),
            WhereWasIPlugin::default(),
            FlyCameraPlugin,
        ))
        .insert_resource(WireframeConfig {
            default_color: WHITE.into(),
            ..default()
        })
        .add_plugins(WireframePlugin::default())
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, PlanetMaterial>,
        >::default())
        .add_systems(
            Startup,
            (
                build_mesh_cache,
                init_skybox,
                build_terrain_tile,
                setup_camera,
            ),
        )
        .add_systems(Update, update_skybox)
        .add_systems(Update, update_terrain_quadtree)
        .add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Transform::from_xyz(100.0, 100.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        DirectionalLight::default(),
    ));
    commands.spawn((
        // Projection::Perspective(PerspectiveProjection {
        //     fov: core::f32::consts::PI / 4.0,
        //     near: 10e-6,
        //     far: 10.0,
        //     aspect_ratio: 1.0,
        // }),
        Hdr,
        Camera3d::default(),
        FlyCam,
        WhereWasI::from_name("terrain_camera"),
        Transform::from_xyz(500.0, 500.0, 500.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
