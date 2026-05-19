use std::collections::VecDeque;

use bevy::ecs::{
    entity::Entity,
    query::Without,
    resource::Resource,
    system::{Query, ResMut},
};
use bevy_terrain::quadtree::ChunkLoaded;

use crate::chunk::Chunk;

const MAX_PERFORMANCE_HISTORY: usize = 128;

#[derive(Resource, Default)]
pub struct OSMPerformance {
    pub chunks_loading: VecDeque<usize>,
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
