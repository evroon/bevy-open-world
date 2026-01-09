use bevy::prelude::*;
use bevy_egui::{
    EguiContexts,
    egui::{self, Color32, Label, Pos2, Ui},
};

use crate::config::ADSBConfig;

#[expect(dead_code)]
fn color_picker(title: &str, color: &mut Vec4, ui: &mut Ui) {
    let mut col = Color32::from_rgb(
        (color[0] * 255.0) as u8,
        (color[1] * 255.0) as u8,
        (color[2] * 255.0) as u8,
    );
    ui.add(Label::new(title));
    ui.end_row();
    if egui::color_picker::color_picker_color32(ui, &mut col, egui::color_picker::Alpha::Opaque) {
        color[0] = col[0] as f32 / 255.0;
        color[1] = col[1] as f32 / 255.0;
        color[2] = col[2] as f32 / 255.0;
    }
    ui.end_row();
}

pub fn adsb_ui(config: &mut ADSBConfig, ui: &mut Ui) {
    // ui.add(
    //     egui::Slider::new(&mut config.emission_strength, 1.0..=20_000.0).text("Emission strength"),
    // );
    // ui.end_row();
    // ui.add(egui::Slider::new(&mut config.emission_threshold, 0.0..=1.0).text("Emission threshold"));
    // ui.end_row();
    ui.add(Label::new(format!(
        "Current time: {}",
        config.time.format("%H:%M:%S")
    )));
    ui.end_row();
}

pub fn ui_system(
    mut adsb_config: ResMut<ADSBConfig>,
    mut contexts: EguiContexts,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::KeyY) {
        adsb_config.ui_visible = !adsb_config.ui_visible;
    }

    if adsb_config.ui_visible {
        egui::Window::new("ADS-B configuration")
            .current_pos(Pos2 { x: 10.0, y: 320.0 })
            .show(contexts.ctx_mut().unwrap(), |ui| {
                egui::Grid::new("3dworld_grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        adsb_ui(adsb_config.as_mut(), ui);
                    });
            });
    }
}
