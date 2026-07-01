use bevy::prelude::*;
use bevy_egui::{
    EguiContexts,
    egui::{self, ComboBox, Label, Pos2, Response, Ui},
};
use bevy_terrain::quadtree::QuadTree;
use egui_plot::Plot;
use egui_plot::PlotPoint;
use egui_plot::PlotPoints;
use egui_plot::{Legend, Points};

use crate::{
    cache::ensure_session_is_valid,
    chunk::{get_root_chunk_for_location, world_to_lat_lon},
    config::{OSMConfig, RasterTileSource},
    location::Location,
    performance::OSMPerformance,
};

fn show_chunks_loading_plot(ui: &mut egui::Ui, performance: &OSMPerformance) -> Response {
    Plot::new("Chunks loading")
        .legend(Legend::default())
        .default_y_bounds(0.0, 500.0)
        .show(ui, |plot_ui| {
            plot_ui.points(
                Points::new(
                    "Chunks loading",
                    PlotPoints::Owned(
                        performance
                            .chunks_loading
                            .iter()
                            .enumerate()
                            .map(|(i, el)| PlotPoint::new(i as f64, *el as f64))
                            .collect(),
                    ),
                )
                .stems(-1.5)
                .radius(1.0)
                .color(performance.get_chunks_loading_plot_color())
                .name("Chunks loading"),
            );
        })
        .response
}

fn show_fps_plot(ui: &mut egui::Ui, performance: &OSMPerformance) -> Response {
    Plot::new("FPS")
        .legend(Legend::default())
        .default_y_bounds(0.0, 120.0)
        .show(ui, |plot_ui| {
            plot_ui.points(
                Points::new(
                    "FPS",
                    PlotPoints::Owned(
                        performance
                            .fps
                            .iter()
                            .enumerate()
                            .map(|(i, el)| PlotPoint::new(i as f64, *el))
                            .collect(),
                    ),
                )
                .stems(-1.5)
                .radius(1.0)
                .color(performance.get_fps_plot_color())
                .name(format!(
                    "FPS (Hz): {:.0}",
                    performance.fps.iter().sum::<f64>() / (performance.fps.len() as f64)
                )),
            );
        })
        .response
}

fn osm_ui(
    commands: &mut Commands,
    config: &mut OSMConfig,
    ui: &mut Ui,
    camera: &Transform,
    quadtrees: &mut Query<(Entity, &mut QuadTree)>,
) {
    let mut selected = config.location.clone();
    ComboBox::from_label("Location")
        .selected_text(selected.get_name())
        .show_ui(ui, |ui| {
            for source in [
                Location::Amsterdam,
                Location::London,
                Location::Monaco,
                Location::NewYork,
            ] {
                ui.selectable_value(&mut selected, source.clone(), source.get_name());
            }
        });
    ui.end_row();

    if selected != config.location {
        info!("Setting location to `{}`", selected.get_name());
        config.location = selected;

        for (entity, mut quadtree) in quadtrees.iter_mut() {
            quadtree.root.destruct(&entity, commands);
            quadtree.root = get_root_chunk_for_location(&config.location);
        }
        ensure_session_is_valid(&config.raster_tile_source);
    }

    let mut selected = config.raster_tile_source.clone();
    ComboBox::from_label("Raster tile source")
        .selected_text(selected.get_name())
        .show_ui(ui, |ui| {
            for source in [
                RasterTileSource::Debug,
                RasterTileSource::CesiumGoogleSatellite,
                RasterTileSource::CesiumGoogleRoadmaps,
                RasterTileSource::CesiumGoogleContour,
                RasterTileSource::OSMDefault,
                RasterTileSource::Transport,
            ] {
                ui.selectable_value(&mut selected, source.clone(), source.get_name());
            }
        });
    ui.end_row();

    if selected != config.raster_tile_source {
        config.raster_tile_source = selected;
        for (entity, mut quadtree) in quadtrees.iter_mut() {
            quadtree.root.destruct(&entity, commands);
        }
        ensure_session_is_valid(&config.raster_tile_source);
    }

    ui.add(Label::new("translation:"));
    ui.add(Label::new(format!(
        "{:.0}, {:.0}, {:.0}",
        camera.translation.x, camera.translation.y, camera.translation.z
    )));
    ui.end_row();
    ui.add(Label::new("lat, lon:"));
    let (lat, lon) = world_to_lat_lon(camera.translation, config.location.get_world_center());
    ui.add(Label::new(format!("{:.5}, {:.5}", lat, lon)));
    ui.end_row();
}

pub fn setup_osm_ui(
    mut commands: Commands,
    mut osm_config: ResMut<OSMConfig>,
    camera: Single<&Transform, With<Camera>>,
    mut contexts: EguiContexts,
    keys: Res<ButtonInput<KeyCode>>,
    mut quadtrees: Query<(Entity, &mut QuadTree)>,
    performance: Res<OSMPerformance>,
) {
    if keys.just_pressed(KeyCode::KeyY) {
        osm_config.ui_visible = !osm_config.ui_visible;
    }

    if osm_config.ui_visible {
        egui::Window::new("OSM configuration")
            .current_pos(Pos2 { x: 10.0, y: 320.0 })
            .show(contexts.ctx_mut().unwrap(), |ui| {
                egui::Grid::new("3dworld_grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        osm_ui(
                            &mut commands,
                            osm_config.as_mut(),
                            ui,
                            &camera,
                            &mut quadtrees,
                        );
                    });
                egui::Grid::new("plot_grid")
                    .num_columns(1)
                    .spacing([100.0, 4.0])
                    .min_row_height(200.0)
                    .striped(true)
                    .show(ui, |ui| {
                        show_chunks_loading_plot(ui, &performance);
                        ui.end_row();
                        show_fps_plot(ui, &performance);
                        ui.end_row();
                    });
            });
    }
}
