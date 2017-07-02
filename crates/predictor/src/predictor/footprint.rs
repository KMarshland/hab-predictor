use rand;
use rand::distributions::{Normal, IndependentSample};
use serde_json;
use predictor::point::*;
use predictor::predictor::*;
use chrono::Duration;

/*
 * All parameters that get passed into the footprint calculation
 */
pub struct FootprintParams {
    pub launch: Point,

    pub burst_altitude_mean : f32,
    pub burst_altitude_std_dev : f32,

    pub ascent_rate_mean : f32,
    pub ascent_rate_std_dev : f32,

    pub descent_rate_mean : f32,
    pub descent_rate_std_dev : f32,

    pub trials: u32
}

#[derive(Serialize)]
pub struct Footprint {
    positions: Vec<Point>
}

impl Footprint {
    pub fn serialize(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

pub fn calculate_footprint(params : FootprintParams) -> Result<Footprint, String> {
    let mut positions : Vec<Point> = vec![];

    let burst_distribution = Normal::new(params.burst_altitude_mean as f64, params.burst_altitude_std_dev as f64);
    let ascent_distribution = Normal::new(params.ascent_rate_mean as f64, params.ascent_rate_std_dev as f64);
    let descent_distribution = Normal::new(params.descent_rate_mean as f64, params.descent_rate_std_dev as f64);

    for _ in 0..params.trials {
        let result = predict(PredictorParams {
            launch: params.launch.clone(),
            profile: PredictionProfile::Standard,

            burst_altitude: burst_distribution.ind_sample(&mut rand::thread_rng()) as f32,
            ascent_rate: ascent_distribution.ind_sample(&mut rand::thread_rng()) as f32,
            descent_rate: descent_distribution.ind_sample(&mut rand::thread_rng()) as f32,

            duration: Duration::minutes(0)
        });

        match result {
            Ok(unwrapped) => {
                match unwrapped {
                    Prediction::Standard(prediction) => {
                        let mut borrowed = prediction;
                        match borrowed.descent.pop() {
                            Some(point) => {
                                positions.push(point)
                            },
                            _ => {}
                        }
                    },
                    _ => {
                        panic!("Yikes");
                    }
                }
            },
            Err(why) => {
                return Err(why);
            }
        }
    }

    Ok(Footprint {
        positions: positions
    })
}
