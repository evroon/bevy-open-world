use bevy::prelude::*;
use bevy_egui::{
    EguiContexts,
    egui::{self, ComboBox, Label, Pos2, Ui},
};
use bevy_terrain::quadtree::{ChunkLoaded, QuadTree};

use crate::{
    chunk::{Chunk, world_to_lat_lon},
    config::{OSMConfig, RasterTileSource},
};

pub fn osm_ui(
    commands: &mut Commands,
    config: &mut OSMConfig,
    ui: &mut Ui,
    camera: &Transform,
    quadtrees: &mut Query<(Entity, &mut QuadTree)>,
    loading_chunks: Query<(Entity, &Chunk), Without<ChunkLoaded>>,
) {
    let mut selected = config.raster_tile_source.clone();
    ComboBox::from_label("Raster tile source")
        .selected_text(format!("{:?}", &selected))
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut selected, RasterTileSource::Debug, "Debug");
            ui.selectable_value(
                &mut selected,
                RasterTileSource::CesiumGoogle,
                "Cesium - Google",
            );
            ui.selectable_value(&mut selected, RasterTileSource::OSMDefault, "OSM - Default");
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
    ui.add(Label::new(format!(
        "Chunks loading: {}",
        loading_chunks.iter().len()
    )));
    ui.end_row();

    if selected != config.raster_tile_source {
        config.raster_tile_source = selected;
        for (entity, mut quadtree) in quadtrees.iter_mut() {
            quadtree.root.destruct(&entity, commands);
        }
    }
}

pub fn setup_osm_ui(
    mut commands: Commands,
    mut osm_config: ResMut<OSMConfig>,
    camera: Single<&Transform, With<Camera>>,
    mut contexts: EguiContexts,
    keys: Res<ButtonInput<KeyCode>>,
    mut quadtrees: Query<(Entity, &mut QuadTree)>,
    loading_chunks: Query<(Entity, &Chunk), Without<ChunkLoaded>>,
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
                            loading_chunks,
                        );
                    });
            });
    }
}
