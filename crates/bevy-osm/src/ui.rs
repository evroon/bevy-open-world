use bevy::prelude::*;
use bevy_egui::{
    EguiContexts,
    egui::{self, ComboBox, Pos2, Ui},
};

use crate::config::{OSMConfig, RasterTileSource};

pub fn osm_ui(config: &mut OSMConfig, ui: &mut Ui) {
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

    if selected != config.raster_tile_source {
        config.raster_tile_source = selected;
    }
}

pub fn setup_osm_ui(
    mut osm_config: ResMut<OSMConfig>,
    mut contexts: EguiContexts,
    keys: Res<ButtonInput<KeyCode>>,
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
                        osm_ui(osm_config.as_mut(), ui);
                    });
            });
    }
}
