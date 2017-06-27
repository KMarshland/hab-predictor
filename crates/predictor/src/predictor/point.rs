use chrono::prelude::*;
use chrono::Duration;
use std::ops::Add;
use std::ops::Mul;
use std::f32;
use serde::ser::{SerializeMap};

const INTEGRAL_DURATION : f32 = 60.0; // seconds
const EARTH_RADIUS : f32 = 6378000.0; // in m
const DATA_RESOLUTION : f32 = 0.5; // resolution in GRIB files

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
 * A point aligned to the resolution in the GRIB file
 */
pub struct AlignedPoint {
    pub latitude: f32,
    pub longitude: f32,
    pub level: i32
}

/*
 * Directions in which a point can be aligned
 */
pub struct Alignment {
    pub ne : AlignedPoint,
    pub nw : AlignedPoint,
    pub se : AlignedPoint,
    pub sw : AlignedPoint,

    pub percent_north : f32,
    pub percent_south : f32,
    pub percent_east : f32,
    pub percent_west : f32
}

/*
 * Velocity data
 */
#[derive(Clone)]
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

    /*
     * Converts the point to an aligned point
     */
    pub fn align(&self) -> Alignment {
        let isobaric_hpa = 1013.25*(1.0 - self.altitude/44330.0).powf(5.255);

        //TODO: make a fast lookup structure for this
        let levels = [2, 3, 5, 7, 10, 20, 30, 50, 70, 100, 150, 200, 250, 300, 350, 400, 450, 500, 550, 600, 650, 700, 750, 800, 850, 900, 925, 950, 975, 1000];
        let mut best_level : i32 = 1;
        let mut best_level_diff : f32 = (isobaric_hpa - (best_level as f32)).abs();

        for level_ref in levels.iter() {
            let level = *level_ref as i32;
            let diff = (isobaric_hpa - (level as f32)).abs();

            if diff < best_level_diff {
                best_level = level;
                best_level_diff = diff;
            }
        }

        // Round to directional DATA_RESOLUTION
        let mangled_lat = self.latitude / DATA_RESOLUTION;
        let mangled_lon = self.longitude / DATA_RESOLUTION;

        let percent_north = mangled_lat.ceil() - mangled_lat.floor();
        let percent_east = mangled_lon.ceil() - mangled_lon.floor();

        Alignment {
            ne: AlignedPoint {
                latitude: Point::align_lat(mangled_lat.ceil()),
                longitude: Point::align_lon(mangled_lon.ceil()),
                level: best_level
            },
            nw: AlignedPoint {
                latitude: Point::align_lat(mangled_lat.ceil()),
                longitude: Point::align_lon(mangled_lon.floor()),
                level: best_level
            },
            se: AlignedPoint {
                latitude: Point::align_lat(mangled_lat.floor()),
                longitude: Point::align_lon(mangled_lon.ceil()),
                level: best_level
            },
            sw: AlignedPoint {
                latitude: Point::align_lat(mangled_lat.floor()),
                longitude: Point::align_lon(mangled_lon.floor()),
                level: best_level
            },
            percent_north: percent_north,
            percent_south: 1.0 - percent_north,
            percent_east: percent_east,
            percent_west: 1.0 - percent_east
        }

    }

    fn align_lat(rounded : f32) -> f32 {
        let mut lat = rounded * DATA_RESOLUTION;

        if lat > 90.0 {
            lat = 90.0;
        } else if lat < -90.0 {
            lat = -90.0
        }

        return lat
    }

    fn align_lon(rounded : f32) -> f32 {
        let mut lon = rounded * DATA_RESOLUTION + 180.0;

        if lon >= 360.0 {
            lon -= 360.0;
        } else if lon < 0.0 {
            lon += 360.0;
        }

        return lon
    }
}

impl<'a> Add<&'a Velocity> for Point {
    type Output = Point;

    fn add(self, velocity: &'a Velocity) -> Point {
        Point {
            latitude: {
                self.latitude + (velocity.north*INTEGRAL_DURATION / EARTH_RADIUS) * (180.0 / f32::consts::PI)
            },
            longitude: {
                bound(
                    self.longitude + (velocity.east*INTEGRAL_DURATION / EARTH_RADIUS) *
                        (180.0 / f32::consts::PI) / f32::cos(self.latitude * f32::consts::PI/180.0)
                )
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

impl AlignedPoint {

    pub fn key(&self) -> u32 {
        AlignedPoint::cache_key(self.level, self.latitude, self.longitude)
    }

    /*
     * Hashes the point to a u32
     */
    pub fn cache_key(level : i32, latitude : f32, longitude: f32) -> u32 {
        // give 10 bits each to each part of the key
        // each of these parts is converted to a u32
        // WARNING: if any has a value greater than 1023 this will have cache collisions

        AlignedPoint::mask(level as f32) +
            (AlignedPoint::mask((latitude + 90.0)/DATA_RESOLUTION) << 10) +
            (AlignedPoint::mask(longitude/DATA_RESOLUTION) << 20)
    }

    pub fn mask(num : f32) -> u32 {
        let mask : u32 = 0b1111111111;
        (num.trunc() as u32) & mask
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

impl Mul<f32> for Velocity {
    type Output = Velocity;

    fn mul(self, factor: f32) -> Velocity {
        Velocity {
            north: self.north * factor,
            east: self.east * factor,
            vertical: self.vertical * factor
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
