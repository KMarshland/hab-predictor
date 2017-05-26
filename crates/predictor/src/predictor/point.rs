use chrono::prelude::*;
use chrono::Duration;
use std::ops::Add;
use std::f32;
use serde::ser::{SerializeMap};

const INTEGRAL_DURATION : f32 = 60.0; // seconds
const EARTH_RADIUS : f32 = 6378000.0; // in m

/*
 * A position time tuple
 */
#[derive(Clone)]
pub struct Point {
    pub latitude: f32,
    pub longitude: f32,
    pub altitude: f32,
    pub time: DateTime<UTC>
}

impl ::serde::Serialize for Point {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: ::serde::Serializer {
        let mut map = serializer.serialize_map(Some(4 as usize))?;
        map.serialize_entry("latitude", &self.latitude)?;
        map.serialize_entry("longitude", &self.longitude)?;
        map.serialize_entry("altitude", &self.altitude)?;
        map.serialize_entry("time", &self.time.to_string())?; // potentially switch this to unix epoch?
        map.end()
    }
}

/*
 * Velocity data
 */
pub struct Velocity {
    pub north: f32,
    pub east: f32,
    pub vertical: f32
}

/*
 * A position time tuple, plus velocity
 */
pub struct Ephemeris {
    pub latitude: f32,
    pub longitude: f32,
    pub altitude: f32,
    pub time: DateTime<UTC>,

    pub velocity: Velocity
}

impl Point {
    /*
     * Create an ephemeris for a point
     */
    pub fn with_velocity(&self, velocity: Velocity) -> Ephemeris {
        Ephemeris {
            latitude: self.latitude,
            longitude: self.longitude,
            altitude: self.altitude,
            time: self.time,

            velocity: velocity
        }
    }
}

impl<'a> Add<&'a Velocity> for Point {
    type Output = Point;

    fn add(self, velocity: &'a Velocity) -> Point {
        Point {
            latitude: {
                self.latitude  + (velocity.north*INTEGRAL_DURATION / EARTH_RADIUS) * (180.0 / f32::consts::PI)
            },
            longitude: {
                bound(self.longitude + (velocity.east*INTEGRAL_DURATION / EARTH_RADIUS) *
                    (180.0 / f32::consts::PI) / f32::cos(self.latitude * f32::consts::PI/180.0))
            },
            altitude: {
                self.altitude + velocity.vertical * INTEGRAL_DURATION
            },
            time: {
                self.time + Duration::seconds(INTEGRAL_DURATION as i64)
            }
        }
    }
}

impl<'a> Add<&'a Velocity> for Velocity {
    type Output = Velocity;

    fn add(self, other: &'a Velocity) -> Velocity {
        Velocity {
            north: self.north + other.north,
            east: self.east + other.east,
            vertical: self.vertical + other.vertical
        }
    }
}

/*
 * Circularly clamps value between -180 and 180
 */
fn bound(x : f32) -> f32 {
    let mut val = x;
    while val < -180.0 {
        val += 360.0
    }

    ((val + 180.0) % 360.0) - 180.0
}
