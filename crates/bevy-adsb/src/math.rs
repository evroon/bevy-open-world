use core::f32::consts::PI;
use core::ops::{Add, Mul, Sub};

use bevy::{
    math::{
        Quat, Vec3,
        ops::{asin, atan2, cos, sin},
    },
    transform::components::Transform,
};

// pub struct ICAOCode(str);

#[derive(Clone)]
pub struct Seconds(pub f32);

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct Degrees(pub f32);

impl Degrees {
    fn to_radians(self) -> f32 {
        self.0 / 180.0 * PI
    }

    fn from_radians(radians: f32) -> Self {
        Self(radians / PI * 180.0)
    }
}

impl Sub for Degrees {
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }

    type Output = Self;
}

impl Add for Degrees {
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }

    type Output = Self;
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Coordinate {
    pub longitude: Degrees,
    pub latitude: Degrees,
}

impl Add for Coordinate {
    fn add(self, rhs: Self) -> Self {
        Coordinate {
            longitude: self.longitude + rhs.longitude,
            latitude: self.latitude + rhs.latitude,
        }
    }

    type Output = Self;
}

impl Sub for Coordinate {
    fn sub(self, rhs: Self) -> Self {
        Coordinate {
            longitude: self.longitude - rhs.longitude,
            latitude: self.latitude - rhs.latitude,
        }
    }

    type Output = Self;
}

impl Mul<f32> for Coordinate {
    fn mul(self, rhs: f32) -> Self {
        Coordinate {
            longitude: Degrees(self.longitude.0 * rhs),
            latitude: Degrees(self.latitude.0 * rhs),
        }
    }

    type Output = Self;
}

/// Calculate point on sphere given longitude and latitude (in radians), and the radius of the sphere
pub fn coordinate_to_point(coordinate: &Coordinate, radius: f32) -> Vec3 {
    let y = sin(coordinate.latitude.to_radians());
    let r = cos(coordinate.latitude.to_radians()); // radius of 2d circle cut through sphere at 'y'
    let x = sin(coordinate.longitude.to_radians()) * r;
    let z = cos(coordinate.longitude.to_radians()) * r;

    Vec3::new(x, y, z) * radius
}

pub fn point_to_coordinate(point_on_unit_sphere: Vec3) -> Coordinate {
    Coordinate {
        longitude: Degrees::from_radians(atan2(point_on_unit_sphere.x, -point_on_unit_sphere.z)),
        latitude: Degrees::from_radians(asin(point_on_unit_sphere.y)),
    }
}

pub fn get_rotation(translation: Vec3, heading: &Degrees) -> Quat {
    Quat::from_axis_angle(translation.normalize(), -heading.to_radians())
        * Transform::from_translation(translation)
            .looking_at(Vec3::ZERO, Vec3::Y)
            .rotation
}
