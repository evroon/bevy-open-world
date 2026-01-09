use bevy::DefaultPlugins;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_terrain::TerrainPlugin;
use bevy_terrain::system::{UniverseGrid, build_terrain};
use bevy_volumetric_clouds::fly_camera::{FlyCam, FlyCameraPlugin};
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
                        title: "Terrain".to_string(),
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
                .disable::<BigSpaceValidationPlugin>(),
        ))
        .add_plugins(EguiPlugin::default())
        .add_plugins(TerrainPlugin)
        .add_plugins((FlyCameraPlugin, WhereWasIPlugin::default()))
        .add_systems(Startup, build_universe)
        .add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ))
        .run();
}

fn build_universe(mut commands: Commands) {
    commands.spawn_big_space(Grid::new(1.0e-1, 0.0), |universe_grid| {
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
            // WhereWasI::from_name("terrain_example"),
            Transform::from_xyz(1.908292, 00.001, 1.066515).looking_at(Vec3::ZERO, Vec3::Y),
        ));
        universe_grid.spawn_spatial((DirectionalLight::default(),));
        build_terrain(universe_grid, 10.0);
    });
}
