use bevy::DefaultPlugins;
use bevy::color::palettes::css::PURPLE;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::pbr::{ExtendedMaterial, OpaqueRendererMethod};
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_fly_camera::system::{FlyCam, FlyCameraPlugin};
use bevy_planet::PlanetsPlugin;
use bevy_planet::material::PlanetMaterial;
use bevy_planet::mesh::{MeshCache, build_mesh_cache};
use bevy_planet::system::{PlanetGrid, build_planet};
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
                .disable::<BigSpaceValidationPlugin>(),
        ))
        .add_plugins(EguiPlugin::default())
        .add_plugins(PlanetsPlugin)
        .add_plugins((FlyCameraPlugin, WhereWasIPlugin::default()))
        .add_systems(Startup, build_universe)
        .add_systems(
            Update,
            (
                update_skybox_transform,
                update_floating_origin.after(TransformSystems::Propagate),
            ),
        )
        .add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ))
        .run();
}

fn build_universe(
    mut commands: Commands,
    meshes: ResMut<'_, Assets<Mesh>>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, PlanetMaterial>>>,
) {
    let radius = 10.0;

    let material = MeshMaterial3d(materials.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color: PURPLE.into(),
            opaque_render_method: OpaqueRendererMethod::Auto,
            ..default()
        },
        extension: PlanetMaterial {
            planet_radius: radius,
            planet_position: Vec3::ZERO,
            floating_origin: Vec3::ZERO,
        },
    }));
    build_mesh_cache(&mut commands, meshes, material);

    commands.spawn_big_space(Grid::new(radius / 100.0, 0.0), |universe_grid| {
        universe_grid.insert((PlanetGrid(),));
        universe_grid.spawn_spatial((
            Projection::Perspective(PerspectiveProjection {
                near: 10e-8,
                ..default()
            }),
            Camera3d::default(),
            Camera { ..default() },
            FlyCam,
            FloatingOrigin,
            // WhereWasI::from_name("planet_example"),
            Transform::from_xyz(30.0, 30.0, 30.0).looking_at(Vec3::ZERO, Vec3::Y),
        ));
        universe_grid.spawn_spatial((DirectionalLight::default(),));
        build_planet(universe_grid, radius);
    });
}

pub fn update_floating_origin(
    universe: Single<(&FloatingOrigin, &CellCoord)>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, PlanetMaterial>>>,
    mesh_cache: Res<MeshCache>,
) {
    let cell_edge_length = 0.1;
    let floating_origin = Vec3::new(
        (universe.1.x as f64 * cell_edge_length) as f32,
        (universe.1.y as f64 * cell_edge_length) as f32,
        (universe.1.z as f64 * cell_edge_length) as f32,
    );

    let mat = materials
        .get_mut(&mesh_cache.material)
        .expect("MeshCache was not initialized");
    mat.extension.floating_origin = floating_origin;
}
