use core::f32::consts::PI;

use bevy::anti_alias::taa::TemporalAntiAliasing;
use bevy::camera::Exposure;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::light::light_consts::lux;
use bevy::light::{
    AtmosphereEnvironmentMapLight, CascadeShadowConfigBuilder, VolumetricFog, VolumetricLight,
};
use bevy::pbr::{Atmosphere, AtmosphereSettings, ScatteringMedium};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::view::Hdr;

use bevy::pbr::ScreenSpaceReflections;

pub fn setup_lighting_for_open_world(mut commands: Commands) {
    // Sun
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: lux::RAW_SUNLIGHT,
            ..default()
        },
        Transform::from_xyz(0.0, 0.2, -1.0).looking_at(Vec3::ZERO, Vec3::Y),
        VolumetricLight,
        (CascadeShadowConfigBuilder {
            minimum_distance: 1.0,
            maximum_distance: 1e4,
            first_cascade_far_bound: 10.0,
            ..Default::default()
        })
        .build(),
    ));
}

pub fn get_camera_bundle_for_open_world(
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
) -> impl Bundle {
    (
        Camera3d::default(),
        Transform::from_xyz(-2.4, 0.04, 0.0).looking_at(Vec3::Y * 0.1, Vec3::Y),
        Atmosphere::earthlike(scattering_mediums.add(ScatteringMedium::default())),
        AtmosphereSettings::default(),
        Exposure { ev100: 13.0 },
        Tonemapping::AcesFitted,
        Hdr,
        Bloom::NATURAL,
        AtmosphereEnvironmentMapLight::default(),
        VolumetricFog::default(),
        Msaa::Off,
        TemporalAntiAliasing::default(),
        ScreenSpaceReflections::default(),
    )
}

pub fn rotate_sun(
    mut suns: Query<&mut Transform, With<DirectionalLight>>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let mut sun_vert_rot_factor = 0.0;
    let mut sun_hor_rot_factor = 0.0;

    if keys.pressed(KeyCode::KeyH) {
        sun_vert_rot_factor -= 0.1;
    }
    if keys.pressed(KeyCode::KeyJ) {
        sun_vert_rot_factor += 0.1;
    }
    if keys.pressed(KeyCode::KeyK) {
        sun_hor_rot_factor -= 0.2;
    }
    if keys.pressed(KeyCode::KeyL) {
        sun_hor_rot_factor += 0.2;
    }

    suns.iter_mut().for_each(|mut tf| {
        tf.rotate_x(time.delta_secs() * PI * sun_vert_rot_factor);
        tf.rotate_y(time.delta_secs() * PI * sun_hor_rot_factor)
    });
}
