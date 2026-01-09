use bevy::{
    core_pipeline::tonemapping::Tonemapping, pbr::ExtendedMaterial, post_process::bloom::Bloom,
    prelude::*, render::view::Hdr,
};
use bevy_adsb::{
    EARTH_RADIUS,
    clickhouse::DataFetch,
    config::{ADSBConfig, init_adsb},
    task_pool::handle_tasks,
    ui::ui_system,
    update::{move_aircraft, spawn_aircraft},
};
use bevy_earth::{
    EarthConfig,
    system::{EarthMaterialExtension, animate_materials, setup_earth},
};
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use bevy_night_sky::system::{init_skybox, update_skybox};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_where_was_i::{WhereWasI, WhereWasIPlugin};

#[derive(Component)]
struct Sun;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins,))
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, EarthMaterialExtension>,
        >::default())
        .add_plugins((
            PanOrbitCameraPlugin,
            WhereWasIPlugin::default(),
            EguiPlugin::default(),
        ))
        .insert_resource(DataFetch::default())
        .insert_resource(EarthConfig::default())
        .add_systems(
            Startup,
            (setup, init_adsb, setup_earth.after(init_adsb), init_skybox),
        )
        .add_systems(EguiPrimaryContextPass, ui_system)
        .add_systems(
            Update,
            (
                move_aircraft,
                update_skybox,
                handle_tasks,
                spawn_aircraft.before(move_aircraft),
                rotate_sun_around_earth,
                animate_materials,
            ),
        )
        .run();
}

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Camera {
            ..Default::default()
        },
        Hdr,
        Tonemapping::TonyMcMapface,
        Bloom::NATURAL,
        PanOrbitCamera {
            zoom_lower_limit: EARTH_RADIUS * 1.1,
            zoom_sensitivity: 0.2,
            pan_sensitivity: 0.0,
            ..Default::default()
        },
        WhereWasI::camera(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Sun
    commands.spawn((
        Sun,
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
    ));
}

fn rotate_sun_around_earth(
    mut suns: Query<&mut Transform, With<Sun>>,
    adsb_config: Res<ADSBConfig>,
    mut earth_config: ResMut<EarthConfig>,
) {
    for mut transform in suns.iter_mut() {
        let sun_dir = adsb_config.get_sun_direction();
        transform.rotation = sun_dir;
        earth_config.sun_direction = sun_dir;
    }
}
