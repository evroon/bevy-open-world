use core::f32::consts::PI;
use std::collections::HashMap;

use bevy::prelude::*;
use chrono::{DateTime, Duration, TimeDelta, Timelike, Utc};

#[derive(Resource, Clone)]
pub struct ADSBConfig {
    pub time: DateTime<Utc>,
    pub buffer_time: DateTime<Utc>,
    pub buffer_ahead_duration: Duration,
    pub buffer_chunk_duration: Duration,
    pub ticks: u32,
    pub target_delta: Duration,
    pub remove_aircraft_after_last_signal: Duration,
    pub history_length: usize,
    pub lookup: HashMap<String, Entity>,
    pub planes: u32,
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub ui_visible: bool,
}

impl ADSBConfig {
    pub fn get_time_of_day(&self) -> f32 {
        self.time.time().num_seconds_from_midnight() as f32 / 86_400.0
    }

    pub fn get_sun_direction(&self) -> Quat {
        Quat::from_rotation_y(self.get_time_of_day() * 2.0 * PI)
    }
}

pub fn init_adsb(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let start = DateTime::parse_from_rfc3339("2025-12-28T00:00:00Z")
        .unwrap()
        .to_utc();

    commands.insert_resource(ADSBConfig {
        planes: 0,
        lookup: HashMap::new(),
        target_delta: TimeDelta::seconds(10),
        remove_aircraft_after_last_signal: TimeDelta::minutes(1),
        history_length: 10,
        ticks: 0,
        buffer_time: start,
        buffer_ahead_duration: Duration::minutes(30),
        buffer_chunk_duration: Duration::minutes(4),
        time: start,
        mesh: meshes.add(Cone::new(0.02, 0.1)),
        material: materials.add(StandardMaterial {
            emissive: LinearRgba::rgb(10.0, 10.0, 100.0),
            ..default()
        }),
        ui_visible: true,
    });
}
