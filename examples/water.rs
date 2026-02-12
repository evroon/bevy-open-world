use bevy::DefaultPlugins;
use bevy::anti_alias::taa::TemporalAntiAliasing;
use bevy::asset::AssetMetaCheck;
use bevy::camera::Exposure;
use bevy::color::palettes::css::WHITE;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::light::{AtmosphereEnvironmentMapLight, VolumetricFog};
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::pbr::{Atmosphere, AtmosphereSettings, ExtendedMaterial, ScatteringMedium};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::{RenderCreation, WgpuFeatures, WgpuSettings};
use bevy::render::view::Hdr;
use bevy_egui::EguiPlugin;
use bevy_terrain::build_terrain_tile;
use bevy_terrain::material::PlanetMaterial;
use bevy_terrain::mesh::build_mesh_cache;
use bevy_terrain::system::update_terrain_quadtree;
use bevy_terrain::water::{AppSettings, Water, adjust_app_settings, spawn_water};
use bevy_volumetric_clouds::fly_camera::{FlyCam, FlyCameraPlugin};
use bevy_where_was_i::{WhereWasI, WhereWasIPlugin};

use bevy::pbr::ScreenSpaceReflections;

/// A marker component for the cube model.
#[derive(Component)]
struct CubeModel;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::linear_rgb(0.4, 0.4, 0.4)))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bevy Open World".to_string(),
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
        .init_resource::<AppSettings>()
        .add_plugins(MaterialPlugin::<ExtendedMaterial<StandardMaterial, Water>>::default())
        .add_plugins(WireframePlugin::default())
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, PlanetMaterial>,
        >::default())
        .add_systems(
            Startup,
            (
                build_mesh_cache,
                // init_skybox,
                build_terrain_tile,
                setup_camera_fog,
                setup_water,
            ),
        )
        // .add_systems(Update, update_skybox)
        .add_systems(Update, update_terrain_quadtree)
        .add_systems(Update, adjust_app_settings)
        .add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ))
        .run();
}

// Spawns the rotating cube.
fn spawn_cube(
    commands: &mut Commands,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    standard_materials: &mut Assets<StandardMaterial>,
) {
    commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(standard_materials.add(StandardMaterial {
                base_color: Color::from(WHITE),
                base_color_texture: Some(asset_server.load("branding/icon.png")),
                ..default()
            })),
            Transform::from_xyz(0.0, 0.5, 0.0),
        ))
        .insert(CubeModel);
}

// Set up the scene.
fn setup_water(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut water_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, Water>>>,
    asset_server: Res<AssetServer>,
) {
    spawn_cube(
        &mut commands,
        &asset_server,
        &mut meshes,
        &mut standard_materials,
    );
    spawn_water(
        &mut commands,
        &asset_server,
        &mut meshes,
        &mut water_materials,
    );
}

fn setup_camera_fog(
    mut commands: Commands,
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
) {
    commands.spawn((
        Transform::from_xyz(100.0, 100.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        DirectionalLight::default(),
    ));
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.4, 0.04, 0.0).looking_at(Vec3::Y * 0.1, Vec3::Y),
        // Earthlike atmosphere
        Atmosphere::earthlike(scattering_mediums.add(ScatteringMedium::default())),
        // Can be adjusted to change the scene scale and rendering quality
        AtmosphereSettings::default(),
        // The directional light illuminance used in this scene
        // (the one recommended for use with this feature) is
        // quite bright, so raising the exposure compensation helps
        // bring the scene to a nicer brightness range.
        Exposure { ev100: 13.0 },
        // Tonemapper chosen just because it looked good with the scene, any
        // tonemapper would be fine :)
        Tonemapping::AcesFitted,
        Hdr,
        // Bloom gives the sun a much more natural look.
        Bloom::NATURAL,
        // Enables the atmosphere to drive reflections and ambient lighting (IBL) for this view
        AtmosphereEnvironmentMapLight::default(),
        FlyCam,
        VolumetricFog {
            ambient_intensity: 0.0,
            ..default()
        },
        WhereWasI::from_name("water_camera"),
        Msaa::Off,
        TemporalAntiAliasing::default(),
        ScreenSpaceReflections::default(),
    ));
}
