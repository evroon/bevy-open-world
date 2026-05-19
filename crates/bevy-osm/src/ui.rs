use std::collections::VecDeque;

use bevy::prelude::*;
use bevy_egui::{
    EguiContexts,
    egui::{self, Color32, ComboBox, Label, Pos2, Response, Ui},
};
use bevy_terrain::quadtree::QuadTree;
use egui_plot::Plot;
use egui_plot::PlotPoint;
use egui_plot::PlotPoints;
use egui_plot::{Legend, Points};

use crate::{
    cache::ensure_session_is_valid,
    chunk::world_to_lat_lon,
    config::{OSMConfig, RasterTileSource},
    performance::OSMPerformance,
};

fn show_plot(ui: &mut egui::Ui, points: &VecDeque<usize>) -> Response {
    Plot::new("Chunks loading")
        .legend(Legend::default())
        .x_axis_label("#chunks")
        .default_y_bounds(0.0, 500.0)
        .show(ui, |plot_ui| {
            plot_ui.points(
                Points::new(
                    "Chunks loading",
                    PlotPoints::Owned(
                        points
                            .iter()
                            .enumerate()
                            .map(|(i, el)| PlotPoint::new(i as f64, *el as f64))
                            .collect(),
                    ),
                )
                .stems(-1.5)
                .radius(1.0)
                .color(Color32::PURPLE)
                .name("chunks_loading"),
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

    if selected != config.raster_tile_source {
        config.raster_tile_source = selected;
        for (entity, mut quadtree) in quadtrees.iter_mut() {
            quadtree.root.destruct(&entity, commands);
        }
        ensure_session_is_valid(&config.raster_tile_source);
    }
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
                show_plot(ui, &performance.chunks_loading);
            });
    }
}
