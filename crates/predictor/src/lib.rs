#[macro_use]
extern crate helix;

#[macro_use]
extern crate lazy_static;
extern crate chrono;

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

extern crate libc;

use chrono::prelude::*;

pub mod predictor;

ruby! {
    class Predictor {
        def test(path: String){
            println!("{}", path)
        }

        def predict(latitude: f64, longitude: f64, altitude: f64, time: String, profile: String, burst_altitude: f64, ascent_rate: f64, descent_rate: f64, duration: f64) -> String {

            predictor::predictor::predict(predictor::predictor::PredictorParams {
                launch: predictor::point::Point {
                    latitude: latitude as f32,
                    longitude: longitude as f32,
                    altitude: altitude as f32,
                    time: {
                        UTC.datetime_from_str(time.as_str(), "%s").unwrap()
                    }
                },

                profile: {
                    match profile.as_ref() {
                        "standard" => predictor::predictor::PredictionProfile::Standard,
                        "valbal" => predictor::predictor::PredictionProfile::ValBal,
                        _ => predictor::predictor::PredictionProfile::Standard
                    }
                },

                burst_altitude: burst_altitude as f32,
                ascent_rate: ascent_rate as f32,
                descent_rate: descent_rate as f32,

                duration: duration as f32
            }).unwrap().serialize()

        }
    }
}
