use std::collections::BinaryHeap;

use bevy::prelude::*;
use chrono::{DateTime, Utc};

use crate::{
    EARTH_RADIUS,
    math::{Coordinate, Degrees, coordinate_to_point},
};

#[derive(Component, Clone, Copy, PartialEq)]
pub struct AircraftState {
    pub coordinate: Coordinate,
    pub heading: Degrees,
    pub ground_speed: f32,
    pub altitude: f32,
    pub timestamp: DateTime<Utc>,
}

impl Eq for AircraftState {}

impl PartialOrd for AircraftState {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AircraftState {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        (self.timestamp).cmp(&(other.timestamp))
    }

    fn max(self, other: Self) -> Self {
        if other.timestamp < self.timestamp {
            self
        } else {
            other
        }
    }

    fn min(self, other: Self) -> Self {
        if other.timestamp > self.timestamp {
            self
        } else {
            other
        }
    }

    fn clamp(self, min: Self, max: Self) -> Self {
        if self.timestamp < min.timestamp {
            min
        } else if self.timestamp > max.timestamp {
            max
        } else {
            self
        }
    }
}

impl AircraftState {
    pub fn get_position_in_world(self) -> Vec3 {
        coordinate_to_point(&self.coordinate, EARTH_RADIUS + self.altitude)
    }
}

#[derive(Component, Clone)]
pub struct Aircraft {
    pub icao: String,
    pub buffer: BinaryHeap<AircraftState>,
    pub last_state: AircraftState,
    pub last_update: DateTime<Utc>,
}

impl Aircraft {
    pub fn seek_buffer(&mut self, current_time: DateTime<Utc>) {
        while let Some(state) = self.buffer.peek() {
            if state.timestamp < current_time {
                self.last_state = self.buffer.pop().unwrap();
            } else {
                break;
            }
        }
    }
}
