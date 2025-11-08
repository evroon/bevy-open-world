use bevy::DefaultPlugins;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_fly_camera::system::{FlyCam, FlyCameraPlugin};
use bevy_planet::PlanetsPlugin;
use bevy_planet::system::{UniverseGrid, build_planet};
use bevy_volumetric_clouds::skybox::system::update_skybox_transform;
use bevy_where_was_i::WhereWasIPlugin;

use big_space::plugin::BigSpaceValidationPlugin;
use big_space::prelude::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::linear_rgb(0.4, 0.4, 0.4)))
        .add_plugins((
            DefaultPlugins
                .build()
                .disable::<TransformPlugin>()
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Planet".to_string(),
                        resolution: (1920, 1040).into(),
                        canvas: Some("#bevy".to_owned()),
                        prevent_default_event_handling: false,
                        fit_canvas_to_parent: true,
                        ..default()
                    }),

                    ..default()
                }),
            BigSpaceDefaultPlugins
                .build()
                .enable::<BigSpaceValidationPlugin>(),
        ))
        .add_plugins(EguiPlugin::default())
        .add_plugins(PlanetsPlugin)
        .add_plugins((FlyCameraPlugin, WhereWasIPlugin::default()))
        .add_systems(Startup, build_universe)
        .add_systems(Update, update_skybox_transform)
        .add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ))
        .run();
}

fn build_universe(mut commands: Commands) {
    commands.spawn_big_space(Grid::new(1.0e-1f32, 0.0), |universe_grid| {
        universe_grid.insert((UniverseGrid(),));
        universe_grid.spawn_spatial((
            Projection::Perspective(PerspectiveProjection {
                fov: core::f32::consts::PI / 4.0,
                near: 10e-6,
                far: 10.0,
                aspect_ratio: 1.0,
            }),
            Camera3d::default(),
            Camera { ..default() },
            FlyCam,
            FloatingOrigin,
            // WhereWasI::from_name("planet_example"),
            Transform::from_xyz(1.908292, 00.001, 1.066515).looking_at(Vec3::ZERO, Vec3::Y),
        ));
        universe_grid.spawn((
            Transform::from_xyz(0.0, -100.0, 0.0).looking_at(Vec3::ZERO, Vec3::X),
            DirectionalLight::default(),
        ));
        build_planet(universe_grid, 10.0);
    });
}
