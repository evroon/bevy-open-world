use std::collections::VecDeque;

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    ecs::{
        entity::Entity,
        query::Without,
        resource::Resource,
        system::{Query, Res, ResMut},
    },
};
use bevy_egui::egui::Color32;
use bevy_terrain::quadtree::ChunkLoaded;

use crate::chunk::Chunk;

const MAX_PERFORMANCE_HISTORY: usize = 128;

#[derive(Resource)]
pub struct OSMPerformance {
    pub chunks_loading: VecDeque<usize>,
    pub fps: VecDeque<f64>,
    pub frametime: VecDeque<f64>,
}

impl Default for OSMPerformance {
    fn default() -> Self {
        Self {
            chunks_loading: vec![0; MAX_PERFORMANCE_HISTORY].into(),
            fps: vec![0.0; MAX_PERFORMANCE_HISTORY].into(),
            frametime: vec![0.0; MAX_PERFORMANCE_HISTORY].into(),
        }
    }
}

impl OSMPerformance {
    pub fn get_chunks_loading_plot_color(&self) -> Color32 {
        let last = *self
            .chunks_loading
            .iter()
            .last()
            .expect("History should always be filled");
        if last > 300 {
            Color32::RED
        } else if last > 100 {
            Color32::ORANGE
        } else {
            Color32::LIGHT_BLUE
        }
    }
    pub fn get_fps_plot_color(&self) -> Color32 {
        let last = *self
            .fps
            .iter()
            .last()
            .expect("History should always be filled");
        if last < 30.0 {
            Color32::RED
        } else if last < 50.0 {
            Color32::ORANGE
        } else {
            Color32::LIGHT_GREEN
        }
    }
}

pub fn update_performance(
    mut performance: ResMut<OSMPerformance>,
    loading_chunks: Query<(Entity, &Chunk), Without<ChunkLoaded>>,
    diagnostics: Res<DiagnosticsStore>,
) {
    performance
        .chunks_loading
        .push_back(loading_chunks.iter().len());

    if let Some(fps) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
    {
        performance.fps.push_back(fps);
    }
    if let Some(frametime) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|fps| fps.smoothed())
    {
        performance.frametime.push_back(frametime);
    }

    while performance.chunks_loading.len() > MAX_PERFORMANCE_HISTORY {
        performance.chunks_loading.pop_front();
    }

    while performance.fps.len() > MAX_PERFORMANCE_HISTORY {
        performance.fps.pop_front();
    }

    while performance.frametime.len() > MAX_PERFORMANCE_HISTORY {
        performance.frametime.pop_front();
    }
}
