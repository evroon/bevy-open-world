use bevy::DefaultPlugins;
use bevy::pbr::{DefaultOpaqueRendererMethod, ScatteringMedium};
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_osm::chunk::Chunk;
use bevy_osm::config::OSMConfig;
use bevy_osm::layer::OMTLayer;
use bevy_osm::material::MapMaterialHandle;
use bevy_osm::tag::Tag;
use bevy_osm::vector::spawn_chunk;
use bevy_terrain::camera::{
    get_camera_bundle_for_open_world, rotate_sun, setup_lighting_for_open_world,
};
use bevy_volumetric_clouds::fly_camera::{FlyCam, FlyCameraPlugin, MovementSettings};
use bevy_where_was_i::{WhereWasI, WhereWasIPlugin};
use lyon::geom::euclid::{Point2D, UnknownUnit};

pub type PolygonInstruction = (Vec<Tag>, OMTLayer, Vec<Point2D<f32, UnknownUnit>>);

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::linear_rgb(0.4, 0.4, 0.4)))
        .insert_resource(DefaultOpaqueRendererMethod::deferred())
        .insert_resource(MovementSettings { speed: 128.0 })
        .insert_resource(OSMConfig::default())
        .add_plugins((
            DefaultPlugins,
            WhereWasIPlugin::default(),
            FlyCameraPlugin,
            EguiPlugin::default(),
        ))
        .init_resource::<MapMaterialHandle>()
        .add_systems(
            Startup,
            (setup_lighting_for_open_world, spawn_camera, spawn_city),
        )
        .add_systems(Update, rotate_sun)
        .run();
}

fn spawn_camera(mut commands: Commands, scattering_mediums: ResMut<Assets<ScatteringMedium>>) {
    let mut camera = commands.spawn(get_camera_bundle_for_open_world(scattering_mediums));
    camera.insert(FlyCam);
    camera.insert(WhereWasI::from_name("city_camera"));
    camera.insert(Projection::Perspective(PerspectiveProjection {
        near: 0.1,
        far: 100.0,
        ..default()
    }));
}

fn spawn_city(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    map_materials: &Res<MapMaterialHandle>,
) {
    let chunk = Chunk {
        x: 8529,
        y: 5974,
        z: 14,
        elevation: Handle::default(),
        raster: Handle::default(),
    };
    let root = commands.spawn(()).id();
    spawn_chunk(commands, meshes, map_materials, &chunk, root);
}
