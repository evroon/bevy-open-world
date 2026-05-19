use std::collections::VecDeque;

use bevy::ecs::{
    entity::Entity,
    query::Without,
    resource::Resource,
    system::{Query, ResMut},
};
use bevy_egui::egui::Color32;
use bevy_terrain::quadtree::ChunkLoaded;

use crate::chunk::Chunk;

const MAX_PERFORMANCE_HISTORY: usize = 128;

#[derive(Resource)]
pub struct OSMPerformance {
    pub chunks_loading: VecDeque<usize>,
}

impl Default for OSMPerformance {
    fn default() -> Self {
        Self {
            chunks_loading: vec![0; MAX_PERFORMANCE_HISTORY].into(),
        }
    }
}

impl OSMPerformance {
    pub fn get_plot_color(&self) -> Color32 {
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
}

pub fn update_performance(
    mut performance: ResMut<OSMPerformance>,
    loading_chunks: Query<(Entity, &Chunk), Without<ChunkLoaded>>,
) {
    performance
        .chunks_loading
        .push_back(loading_chunks.iter().len());

    while performance.chunks_loading.len() > MAX_PERFORMANCE_HISTORY {
        performance.chunks_loading.pop_front();
    }
}
