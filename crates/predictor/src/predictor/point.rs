use chrono::prelude::*;

/*
 * A position time tuple
 */
pub struct Point {
    pub latitude: f32,
    pub longitude: f32,
    pub altitude: f32,
    pub time: DateTime<UTC>
}

/*
 * Velocity data
 */
pub struct Velocity {
    x: f32,
    y: f32,
    z: f32
}

/*
 * A position time tuple, plus velocity
 */
pub struct Ephemeris {
    latitude: f32,
    longitude: f32,
    altitude: f32,
    time: DateTime<UTC>,

    velocity: Velocity
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
