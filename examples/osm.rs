use bevy::DefaultPlugins;
use bevy::pbr::{DefaultOpaqueRendererMethod, ScatteringMedium};
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_flight_sim::runway::spawn_aircraft;
use bevy_osm::elevation::spawn_elevation_mesh;
use bevy_panorbit_camera::PanOrbitCameraPlugin;
use bevy_terrain::WaterPlugin;
use bevy_terrain::camera::{
    get_camera_bundle_for_open_world, rotate_sun, setup_lighting_for_open_world,
};
use bevy_terrain::system::update_terrain_quadtree;
use bevy_terrain::water::spawn_water;
use bevy_volumetric_clouds::fly_camera::{FlyCam, FlyCameraPlugin, MovementSettings};
use bevy_where_was_i::{WhereWasI, WhereWasIPlugin};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::linear_rgb(0.4, 0.4, 0.4)))
        .insert_resource(DefaultOpaqueRendererMethod::deferred())
        .insert_resource(MovementSettings { speed: 10.0 })
        .add_plugins((
            DefaultPlugins,
            // OSMPlugin,
            PanOrbitCameraPlugin,
            WhereWasIPlugin::default(),
            FlyCameraPlugin,
            EguiPlugin::default(),
            WaterPlugin,
        ))
        .add_systems(
            Startup,
            (
                setup_lighting_for_open_world,
                spawn_camera,
                spawn_water,
                spawn_aircraft,
                spawn_elevation_mesh,
            ),
        )
        .add_systems(Update, (update_terrain_quadtree, rotate_sun))
        .run();
}

fn spawn_camera(mut commands: Commands, scattering_mediums: ResMut<Assets<ScatteringMedium>>) {
    let mut camera = commands.spawn(get_camera_bundle_for_open_world(scattering_mediums));
    camera.insert(FlyCam);
    camera.insert(WhereWasI::from_name("osm_camera"));
    camera.insert(Projection::Perspective(PerspectiveProjection {
        near: 0.001,
        far: 10.0,
        ..default()
    }));
}
