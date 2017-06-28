#[macro_use]
extern crate helix;

#[macro_use]
extern crate lazy_static;
extern crate chrono;

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

extern crate lru_cache;
extern crate rand;
extern crate libc;

use chrono::prelude::*;

#[macro_use]
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

                duration: chrono::Duration::seconds(duration as i64)
            }).unwrap().serialize()

        }

        def footprint(latitude: f64, longitude: f64, altitude: f64, time: String, burst_altitude_mean: f64, burst_altitude_std_dev: f64, ascent_rate_mean: f64, ascent_rate_std_dev: f64, descent_rate_mean: f64, descent_rate_std_dev: f64, trials: i64) -> String {

            predictor::footprint::calculate_footprint(predictor::footprint::FootprintParams {
                launch: predictor::point::Point {
                    latitude: latitude as f32,
                    longitude: longitude as f32,
                    altitude: altitude as f32,
                    time: {
                        UTC.datetime_from_str(time.as_str(), "%s").unwrap()
                    }
                },

                burst_altitude_mean: burst_altitude_mean as f32,
                burst_altitude_std_dev: burst_altitude_std_dev as f32,

                ascent_rate_mean: ascent_rate_mean as f32,
                ascent_rate_std_dev: ascent_rate_std_dev as f32,

                descent_rate_mean: descent_rate_mean as f32,
                descent_rate_std_dev: descent_rate_std_dev as f32,

                trials: trials as u32
            }).unwrap().serialize()

        }

        def guidance(latitude: f64, longitude: f64, altitude: f64, time: String, timeout: f64, duration: f64, time_increment: f64, altitude_variance: f64, altitude_increment: f64, compare_with_naive: bool) -> String {
        predictor::guidance::guidance(predictor::guidance::GuidanceParams {
                launch: predictor::point::Point {
                    latitude: latitude as f32,
                    longitude: longitude as f32,
                    altitude: altitude as f32,
                    time: {
                        UTC.datetime_from_str(time.as_str(), "%s").unwrap()
                    }
                },

                duration: chrono::Duration::seconds(duration as i64),
                timeout: timeout as f32,

                time_increment: chrono::Duration::seconds(time_increment as i64),

                altitude_variance: altitude_variance as u32,
                altitude_increment: altitude_increment as u32,

                compare_with_naive: compare_with_naive
            }).unwrap().serialize()
        }

    }
}
