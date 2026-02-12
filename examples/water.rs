use core::f32::consts::PI;

use bevy::DefaultPlugins;
use bevy::anti_alias::taa::TemporalAntiAliasing;
use bevy::asset::AssetMetaCheck;
use bevy::camera::Exposure;
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
        Atmosphere::earthlike(scattering_mediums.add(ScatteringMedium::default())),
        AtmosphereSettings::default(),
        Exposure { ev100: 13.0 },
        Tonemapping::AcesFitted,
        Hdr,
        Bloom::NATURAL,
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

fn dynamic_scene(mut suns: Query<&mut Transform, With<DirectionalLight>>, time: Res<Time>) {
    // Only rotate the sun if motion is not paused
    suns.iter_mut()
        .for_each(|mut tf| tf.rotate_x(-time.delta_secs() * PI / 10.0));
}
