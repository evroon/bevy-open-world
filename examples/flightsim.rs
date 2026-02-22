use bevy::DefaultPlugins;
use bevy::asset::AssetMetaCheck;
use bevy::pbr::{DefaultOpaqueRendererMethod, ScatteringMedium};
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_flight_sim::runway::{spawn_aircraft, spawn_runway};
use bevy_terrain::camera::{
    get_camera_bundle_for_open_world, rotate_sun, setup_lighting_for_open_world,
};
use bevy_terrain::water::spawn_water;
use bevy_terrain::{TerrainPlugin, WaterPlugin};
use bevy_volumetric_clouds::fly_camera::{FlyCam, FlyCameraPlugin, MovementSettings};
use bevy_where_was_i::{WhereWasI, WhereWasIPlugin};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::linear_rgb(0.4, 0.4, 0.4)))
        .insert_resource(DefaultOpaqueRendererMethod::deferred())
        .insert_resource(MovementSettings { speed: 1.0 })
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
            WaterPlugin,
            TerrainPlugin,
        ))
        .add_systems(
            Startup,
            (
                setup_lighting_for_open_world,
                spawn_camera,
                spawn_water,
                spawn_runway,
                spawn_aircraft,
            ),
        )
        .add_systems(Update, rotate_sun)
        .run();
}

fn spawn_camera(mut commands: Commands, scattering_mediums: ResMut<Assets<ScatteringMedium>>) {
    let mut camera = commands.spawn(get_camera_bundle_for_open_world(scattering_mediums));
    camera.insert(FlyCam);
    camera.insert(WhereWasI::from_name("flightsim_camera"));
}
