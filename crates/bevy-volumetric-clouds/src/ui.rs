use bevy::prelude::*;
use bevy_egui::{
    EguiContexts,
    egui::{self, Color32, Pos2, Ui},
};

use super::compute::CloudsConfig;

fn color_picker(title: &str, color: &mut Vec4, ui: &mut Ui) {
    let mut col = Color32::from_rgb(
        (color[0] * 256.0) as u8,
        (color[1] * 256.0) as u8,
        (color[2] * 256.0) as u8,
    );
    ui.add(egui::Label::new(title));
    ui.end_row();
    if egui::color_picker::color_picker_color32(ui, &mut col, egui::color_picker::Alpha::Opaque) {
        color[0] = col[0] as f32 / 256.0;
        color[1] = col[1] as f32 / 256.0;
        color[2] = col[2] as f32 / 256.0;
    }
    ui.end_row();
}

pub fn clouds_ui(config: &mut CloudsConfig, ui: &mut Ui) {
    ui.add(egui::Slider::new(&mut config.march_steps, 1..=100).text("March steps"));
    ui.end_row();
    ui.add(egui::Slider::new(&mut config.self_shadow_steps, 1..=50).text("Self shadow steps"));
    ui.end_row();
    ui.add(egui::Slider::new(&mut config.earth_radius, 5e4..=1e7).text("Earth radius"));
    ui.end_row();
    ui.add(egui::Slider::new(&mut config.bottom, 1.0..=5e3).text("Bottom"));
    ui.end_row();
    ui.add(egui::Slider::new(&mut config.top, 1.0..=5e3).text("Top"));
    ui.end_row();
    ui.add(egui::Slider::new(&mut config.coverage, 0.0..=1.0).text("coverage"));
    ui.end_row();
    ui.add(egui::Slider::new(&mut config.detail_strength, 0.0..=1.0).text("detail_strength"));
    ui.end_row();
    ui.add(egui::Slider::new(&mut config.base_edge_softness, 0.0..=1.0).text("base_edge_softness"));
    ui.end_row();
    ui.add(egui::Slider::new(&mut config.bottom_softness, 0.01..=10.0).text("bottom_softness"));
    ui.end_row();
    ui.add(egui::Slider::new(&mut config.density, 0.001..=1.0).text("density"));
    ui.end_row();
    ui.add(
        egui::Slider::new(&mut config.shadow_march_step_size, 1.0..=100.0)
            .text("shadow_march_step_size"),
    );
    ui.end_row();
    ui.add(
        egui::Slider::new(&mut config.shadow_march_step_multiply, 0.1..=10.0)
            .text("shadow_march_step_multiply"),
    );
    ui.end_row();
    ui.add(
        egui::Slider::new(&mut config.forward_scattering_g, -10.0..=10.0)
            .text("forward_scattering_g"),
    );
    ui.end_row();
    ui.add(
        egui::Slider::new(&mut config.backward_scattering_g, -10.0..=10.0)
            .text("backward_scattering_g"),
    );
    ui.end_row();
    ui.add(egui::Slider::new(&mut config.scattering_lerp, 0.01..=100.0).text("scattering_lerp"));
    ui.end_row();
    ui.add(
        egui::Slider::new(&mut config.min_transmittance, 0.01..=100.0).text("min_transmittance"),
    );
    ui.end_row();
    ui.add(egui::Slider::new(&mut config.base_scale, 0.1..=100.0).text("base_scale"));
    ui.end_row();
    ui.add(egui::Slider::new(&mut config.detail_scale, 1.0..=100.0).text("detail_scale"));
    ui.end_row();
    ui.add(egui::Slider::new(&mut config.camera_fl, 1.0..=10.0).text("camera_fl"));
    ui.end_row();
    ui.add(egui::Slider::new(&mut config.debug, 0.0001..=100.0).text("debug"));
    ui.end_row();
    ui.add(
        egui::Slider::new(&mut config.reprojection_strength, 0.0..=1.0)
            .text("reprojection_strength"),
    );
    ui.end_row();

    color_picker("ambient_color_top", &mut config.ambient_color_top, ui);
    color_picker("ambient_color_bottom", &mut config.ambient_color_bottom, ui);
    color_picker("sun_color", &mut config.sun_color, ui);

    if ui.button("Reset to defaults").clicked() {
        *config = CloudsConfig::default();
    };
}

pub fn ui_system(
    mut clouds_config: ResMut<CloudsConfig>,
    mut contexts: EguiContexts,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::KeyY) {
        clouds_config.ui_visible = !clouds_config.ui_visible;
    }

    if clouds_config.ui_visible {
        egui::Window::new("Clouds")
            .current_pos(Pos2 { x: 10., y: 320. })
            .show(contexts.ctx_mut().unwrap(), |ui| {
                egui::Grid::new("3dworld_grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        clouds_ui(clouds_config.as_mut(), ui);
                    });
            });
    }
}
