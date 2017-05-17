use chrono::prelude::*;

pub struct Point {
    latitude: f32,
    longitude: f32,
    altitude: f32,
    time: DateTime<UTC>
}
