use serde_json;
use predictor::point::*;
use predictor::predictor::*;

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

    for _ in 0..params.trials {
        let result = predict(PredictorParams {
            launch: params.launch.clone(),
            profile: PredictionProfile::Standard,

            burst_altitude: params.burst_altitude_mean,
            ascent_rate: params.ascent_rate_mean,
            descent_rate: params.descent_rate_mean,

            duration: 0.0
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
