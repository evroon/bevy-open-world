use std::{f32::consts::PI, f64::consts::PI as PI_64};

use bevy::math::ops::{atan, powf, sinh};

#[derive(Debug, PartialEq)]
pub struct Chunk {
    pub x: i32,
    pub y: i32,
    pub z: i8,
}
impl Chunk {
    pub fn get_cache_path(&self) -> String {
        let (z, x, y) = (self.z, self.x, self.y);
        format!("assets/elevation/v1/{z}/{x}/{y}")
    }
}

pub fn get_chunk_for_coord(lat_deg: f64, lon_deg: f64, zoom: i8) -> Chunk {
    let n = (1 << zoom) as f64;

    let x_tile = (n * (lon_deg + 180.0) / 360.0) as i32;

    let lat_rad = lat_deg.to_radians();
    let y_tile = (n * (1.0 - (lat_rad.tan() + (1.0 / lat_rad.cos())).ln() / PI_64) / 2.0) as i32;

    Chunk {
        x: x_tile,
        y: y_tile,
        z: zoom,
    }
}

pub fn get_lat_lon(x: f32, y: f32, zoom: i8) -> (f32, f32) {
    (
        atan(sinh(PI - y / powf(2.0, zoom as f32) * 2.0 * PI)) * 180.0 / PI,
        x / powf(2.0, zoom as f32) * 360.0 - 180.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lat_lon_chunk_conversion() {
        let chunk = Chunk {
            x: 140178,
            y: 97411,
            z: 18,
        };

        let (lat, lon) = get_lat_lon(chunk.x as f32, chunk.y as f32, chunk.z);
        assert_eq!((lat, lon), (41.899208, 12.505188));
        assert_eq!(get_chunk_for_coord(lat as f64, lon as f64, chunk.z), chunk);
    }
}
