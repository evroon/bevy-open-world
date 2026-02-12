use core::f32::consts::PI;

use bevy::DefaultPlugins;
use bevy::anti_alias::fxaa::Fxaa;
use bevy::anti_alias::taa::TemporalAntiAliasing;
use bevy::asset::AssetMetaCheck;
use bevy::color::palettes::css::WHITE;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::light::light_consts::lux;
use bevy::light::{
    AtmosphereEnvironmentMapLight, CascadeShadowConfigBuilder, VolumetricFog, VolumetricLight,
};
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::pbr::{
    Atmosphere, AtmosphereSettings, DefaultOpaqueRendererMethod, ExtendedMaterial, ScatteringMedium,
};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::{RenderCreation, WgpuFeatures, WgpuSettings};
use bevy::render::view::Hdr;
use bevy_egui::EguiPlugin;
use bevy_terrain::material::PlanetMaterial;
use bevy_terrain::water::{Water, spawn_water};
use bevy_volumetric_clouds::fly_camera::{FlyCam, FlyCameraPlugin};
use bevy_where_was_i::{WhereWasI, WhereWasIPlugin};

use bevy::pbr::ScreenSpaceReflections;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::linear_rgb(0.4, 0.4, 0.4)))
        .insert_resource(DefaultOpaqueRendererMethod::deferred())
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
        .add_plugins(MaterialPlugin::<ExtendedMaterial<StandardMaterial, Water>>::default())
        .add_plugins(WireframePlugin::default())
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, PlanetMaterial>,
        >::default())
        .add_systems(Startup, (setup_camera_fog, spawn_water))
        .add_systems(Update, dynamic_scene)
        .add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ))
        .run();
}

fn setup_camera_fog(
    mut commands: Commands,
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
) {
    // Configure a properly scaled cascade shadow map for this scene (defaults are too large, mesh units are in km)
    let cascade_shadow_config = CascadeShadowConfigBuilder {
        first_cascade_far_bound: 0.3,
        maximum_distance: 15.0,
        ..default()
    }
    .build();
    // Sun
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            // lux::RAW_SUNLIGHT is recommended for use with this feature, since
            // other values approximate sunlight *post-scattering* in various
            // conditions. RAW_SUNLIGHT in comparison is the illuminance of the
            // sun unfiltered by the atmosphere, so it is the proper input for
            // sunlight to be filtered by the atmosphere.
            illuminance: lux::RAW_SUNLIGHT,
            ..default()
        },
        Transform::from_xyz(1.0, 0.4, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        VolumetricLight,
        cascade_shadow_config,
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
        // Exposure { ev100: 13.0 },
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

// Spawns the camera.
pub fn spawn_camera_for_water(mut commands: Commands) {
    // Create the camera. Add an environment map and skybox so the water has
    // something interesting to reflect, other than the cube. Enable deferred
    // rendering by adding depth and deferred prepasses. Turn on FXAA to make
    // the scene look a little nicer. Finally, add screen space reflections.
    commands
        .spawn((
            Camera3d::default(),
            Transform::from_translation(vec3(-1.25, 2.25, 4.5)).looking_at(Vec3::ZERO, Vec3::Y),
            Hdr,
            Msaa::Off,
        ))
        .insert(ScreenSpaceReflections::default())
        .insert(Fxaa::default());
}

fn dynamic_scene(mut suns: Query<&mut Transform, With<DirectionalLight>>, time: Res<Time>) {
    // Only rotate the sun if motion is not paused
    suns.iter_mut()
        .for_each(|mut tf| tf.rotate_x(-time.delta_secs() * PI / 10.0));
}
