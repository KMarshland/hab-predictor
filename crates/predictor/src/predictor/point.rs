use chrono::prelude::*;
use chrono::Duration;
use serde::ser::{SerializeMap};
use std::ops::Add;
use std::ops::Mul;
use std::fmt;
use std::f32;

const INTEGRAL_DURATION : f32 = 60.0; // seconds
const EARTH_RADIUS : f32 = 6371_000.0; // in m
const DATA_RESOLUTION : f32 = 0.5; // resolution in GRIB files

pub type Temperature = f32;

/*
 * A position time tuple
 */
#[derive(Clone)]
#[derive(Debug)]
pub struct Point {
    pub latitude: f32,
    pub longitude: f32,
    pub altitude: f32,
    pub time: DateTime<Utc>
}

impl fmt::Display for Point {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{} / {}m at {}", self.latitude, self.longitude, self.altitude, self.time.to_string())
    }
}

impl ::serde::Serialize for Point {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: ::serde::Serializer {
        let mut map = serializer.serialize_map(Some(4 as usize))?;
        map.serialize_entry("latitude", &self.latitude)?;
        map.serialize_entry("longitude", &self.longitude)?;
        map.serialize_entry("altitude", &self.altitude)?;
        map.serialize_entry("time", &format!("{:?}", &self.time))?; // potentially switch this to unix epoch?
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
    pub ne_down : AlignedPoint,
    pub ne_up : AlignedPoint,
    pub nw_down : AlignedPoint,
    pub nw_up : AlignedPoint,
    pub se_down : AlignedPoint,
    pub se_up : AlignedPoint,
    pub sw_down : AlignedPoint,
    pub sw_up : AlignedPoint,

    pub percent_north : f32,
    pub percent_south : f32,
    pub percent_east : f32,
    pub percent_west : f32,
    pub percent_down : f32,
    pub percent_up : f32
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
 * Represents atmospheric condition at a given point
 */
#[derive(Clone)]
pub struct Atmospheroid {
    pub temperature : Temperature,
    pub velocity: Velocity
}

impl Point {

    /*
     * Returns the distance to another point in meters
     */
    pub fn distance_to(&self, other : &Point) -> f32 {

        let lat1 = self.latitude.to_radians();
        let lat2 = other.latitude.to_radians();

        let delta_lat = lat1 - lat2;
        let delta_lon = (self.longitude - other.longitude).to_radians();

        let a = (delta_lat/2.0).sin() * (delta_lat/2.0).sin() +
            lat1.cos() * lat2.cos() *
                (delta_lon/2.0).sin() * (delta_lon/2.0).sin();
        let c = 2.0 * a.sqrt().atan2((1.0-a).sqrt());

        c * EARTH_RADIUS
    }

    /*
     * Converts the point to an aligned point
     */
    pub fn align(&self) -> Alignment {
        let isobaric_hpa = 1013.25*(1.0 - self.altitude/44330.0).powf(5.255);

        //TODO: make a fast lookup structure for this
        let levels = [2, 3, 5, 7, 10, 20, 30, 50, 70, 80, 100, 150, 200, 250, 300, 350, 400, 450, 500, 550, 600, 650, 700, 750, 800, 850, 900, 925, 950, 975, 1000];
        let mut best_level : i32 = 1;
        let mut best_level_diff : f32 = (isobaric_hpa - (best_level as f32)).abs();
        let mut best_level_index : usize = 0;
        let mut curr_index : usize = 0;

        for level_ref in levels.iter() {
            let level = *level_ref as i32;
            let diff = (isobaric_hpa - (level as f32)).abs();

            if diff < best_level_diff {
                best_level = level;
                best_level_diff = diff;
                best_level_index = curr_index;
            }
            curr_index = curr_index + 1;
        }

        // Determine which level is up and which is down
        let level_up : i32;
        let level_down : i32;
        let mut level_down_diff : f32 = 0.0;

        let alt_floor = levels[0] as f32;
        let alt_ceil  = levels[1] as f32;

        if isobaric_hpa < alt_floor || isobaric_hpa > alt_ceil {

            level_up = best_level;
            level_down = best_level;

        } else if best_level > (isobaric_hpa as i32) {
            level_up = best_level;

            level_down = levels[best_level_index - 1];
            level_down_diff = (isobaric_hpa - (level_down as f32)).abs();
        } else {
            level_up = levels[best_level_index + 1];

            level_down = best_level;
            level_down_diff = best_level_diff;
        }

        // Round to directional DATA_RESOLUTION
        let mangled_lat = self.latitude / DATA_RESOLUTION;
        let mangled_lon = self.longitude / DATA_RESOLUTION;

        let percent_north = mangled_lat.ceil() - mangled_lat.floor();
        let percent_east = mangled_lon.ceil() - mangled_lon.floor();

        let mut percent_down : f32 = 1.0;
        if level_up != level_down {
            percent_down = level_down_diff / ((level_up - level_down) as f32);
        }

        Alignment {
            ne_down: AlignedPoint {
                latitude: Point::align_lat(mangled_lat.ceil()),
                longitude: Point::align_lon(mangled_lon.ceil()),
                level: level_down
            },
            ne_up: AlignedPoint {
                latitude: Point::align_lat(mangled_lat.ceil()),
                longitude: Point::align_lon(mangled_lon.ceil()),
                level: level_up
            },
            nw_down: AlignedPoint {
                latitude: Point::align_lat(mangled_lat.ceil()),
                longitude: Point::align_lon(mangled_lon.floor()),
                level: level_down
            },
            nw_up: AlignedPoint {
                latitude: Point::align_lat(mangled_lat.ceil()),
                longitude: Point::align_lon(mangled_lon.floor()),
                level: level_up
            },
            se_down: AlignedPoint {
                latitude: Point::align_lat(mangled_lat.floor()),
                longitude: Point::align_lon(mangled_lon.ceil()),
                level: level_down
            },
            se_up: AlignedPoint {
                latitude: Point::align_lat(mangled_lat.floor()),
                longitude: Point::align_lon(mangled_lon.ceil()),
                level: level_up
            },
            sw_down: AlignedPoint {
                latitude: Point::align_lat(mangled_lat.floor()),
                longitude: Point::align_lon(mangled_lon.floor()),
                level: level_down
            },
            sw_up: AlignedPoint {
                latitude: Point::align_lat(mangled_lat.floor()),
                longitude: Point::align_lon(mangled_lon.floor()),
                level: level_up
            },
            percent_north: percent_north,
            percent_south: 1.0 - percent_north,
            percent_east: percent_east,
            percent_west: 1.0 - percent_east,
            percent_down: percent_down,
            percent_up: 1.0 - percent_down
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

    pub fn add_with_duration(self, velocity: &Velocity, integral_duration : f32) -> Point {
        Point {
            latitude: {
                self.latitude + (velocity.north*integral_duration / EARTH_RADIUS) * (180.0 / f32::consts::PI)
            },
            longitude: {
                bound(
                    self.longitude + (velocity.east*integral_duration / EARTH_RADIUS) *
                        (180.0 / f32::consts::PI) / f32::cos(self.latitude * f32::consts::PI/180.0)
                )
            },
            altitude: {
                self.altitude + velocity.vertical * integral_duration
            },
            time: {
                self.time + Duration::seconds(integral_duration as i64)
            }
        }
    }
}

impl<'a> Add<&'a Velocity> for Point {
    type Output = Point;

    fn add(self, velocity: &'a Velocity) -> Point {
        self.add_with_duration(velocity, INTEGRAL_DURATION)
    }
}

impl AlignedPoint {

    pub fn key(&self, dataset_id : u32) -> u32 {
        AlignedPoint::cache_key(self.level, self.latitude, self.longitude, dataset_id)
    }

    /*
     * Hashes the point to a u32
     */
    pub fn cache_key(level : i32, latitude : f32, longitude: f32, dataset_id : u32) -> u32 {
        // give 10 bits each to each part of the key
        // each of these parts is converted to a u32
        // WARNING: if any has a value greater than 1023 this will have cache collisions

        let level_index : u32 = match level {
            2 => 0,
            3 => 1,
            5 => 2,
            7 => 3,
            10 => 4,
            20 => 5,
            30 => 6,
            50 => 7,
            70 => 8,
            80 => 9,
            100 => 10,
            150 => 11,
            200 => 12,
            250 => 13,
            300 => 14,
            350 => 15,
            400 => 16,
            450 => 17,
            500 => 18,
            550 => 19,
            600 => 20,
            650 => 21,
            700 => 22,
            750 => 23,
            800 => 24,
            850 => 25,
            900 => 26,
            925 => 27,
            950 => 28,
            975 => 29,
            1000 => 30,
            _ => {
                panic!(format!("Unknown level for an aligned point: {}", level))
            }
        };

        AlignedPoint::mask5(level_index) +
            AlignedPoint::mask5(dataset_id) << 5 +
            (AlignedPoint::mask10((latitude + 90.0)/DATA_RESOLUTION) << 10) +
            (AlignedPoint::mask10(longitude/DATA_RESOLUTION) << 20)
    }

    fn mask5(num : u32) -> u32 {
        let mask : u32 = 0b11111;
        num & mask
    }

    fn mask10(num : f32) -> u32 {
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

impl<'a> Add<&'a Atmospheroid> for Atmospheroid {
    type Output = Atmospheroid;

    fn add(self, other: &'a Atmospheroid) -> Atmospheroid {
        Atmospheroid {
            temperature: self.temperature + other.temperature,
            velocity: self.velocity + &other.velocity
        }
    }
}

/*
 * Piecewise multiplication, so that it can be scaled for interpolation
 */
impl Mul<f32> for Atmospheroid {
    type Output = Atmospheroid;

    fn mul(self, factor: f32) -> Atmospheroid {
        Atmospheroid {
            velocity: self.velocity * factor,
            temperature: self.temperature * factor
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
