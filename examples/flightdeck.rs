use bevy::DefaultPlugins;
use bevy::dev_tools::infinite_grid::InfiniteGridPlugin;
use bevy::pbr::DefaultOpaqueRendererMethod;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_flight_sim::flightdeck::spawn_flightdeck;
use bevy_flight_sim::runway::spawn_aircraft;
use bevy_osm::config::OSMConfig;
use bevy_osm::{OSMPlugin, location::Location};
use bevy_terrain::camera::{
    get_camera_bundle_for_open_world, rotate_sun, setup_lighting_for_open_world,
};
use bevy_volumetric_clouds::fly_camera::{FlyCam, FlyCameraPlugin, MovementSettings};
use bevy_where_was_i::{WhereWasI, WhereWasIPlugin};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::linear_rgb(0.4, 0.4, 0.4)))
        .insert_resource(DefaultOpaqueRendererMethod::deferred())
        .insert_resource(MovementSettings { speed: 2.0 })
        .insert_resource(OSMConfig {
            location: Location::Monaco,
            ..Default::default()
        })
        .add_plugins((
            DefaultPlugins,
            InfiniteGridPlugin,
            OSMPlugin,
            WhereWasIPlugin::default(),
            FlyCameraPlugin,
            EguiPlugin::default(),
            // WaterPlugin,
        ))
        .add_systems(
            Startup,
            (
                setup_lighting_for_open_world,
                spawn_flightdeck,
                spawn_aircraft,
                spawn_camera,
            ),
        )
        .add_systems(Update, rotate_sun)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    let mut camera = commands.spawn(get_camera_bundle_for_open_world());
    camera.insert(FlyCam);
    camera.insert(WhereWasI::from_name("flightdeck"));
    camera.insert(Projection::Perspective(PerspectiveProjection {
        near: 0.1,
        far: 100.0,
        ..default()
    }));
}
