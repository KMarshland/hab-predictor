use predictor::point::*;
use predictor::dataset_reader::velocity_at;
use chrono::Duration;
use serde_json;

pub enum PredictionProfile {
    Standard,
    Float,
    ValBal
}

/*
 * All parameters that might be passed in to the prediction
 */
pub struct PredictorParams {
    pub launch: Point,

    pub profile: PredictionProfile,

    // standard profile
    pub burst_altitude: f32, // meters
    pub ascent_rate: f32, // meters per second
    pub descent_rate: f32, // meters per second

    // valbal
    pub duration: Duration
}

/*
 * Only those parameters necessary to run a standard profile prediction
 */
struct StandardPredictorParams {
    launch: Point,

    // standard profile
    burst_altitude: f32, // meters
    ascent_rate: f32, // meters per second
    descent_rate: f32, // meters per second
}

/*
 * Only those parameters necessary to run a float/valbal prediction
 */
struct FloatPredictionParams {
    launch: Point,

    // valbal
    duration: Duration
}

/*
 * The result of a prediction
 */
pub enum Prediction {
    Standard(StandardPrediction),
    Float(FloatPrediction),
    ValBal(FloatPrediction)
}

#[derive(Serialize)]
pub struct StandardPrediction {
    pub ascent: Vec<Point>,
    pub burst: Point,
    pub descent: Vec<Point>
}

#[derive(Serialize)]
pub struct FloatPrediction {
    pub positions: Vec<Point>
}

impl Prediction {
    pub fn serialize(&self) -> String {
        match *self {
            Prediction::Standard(ref p) => {
                serde_json::to_string(p).unwrap()
            },
            Prediction::Float(ref p) | Prediction::ValBal(ref p) => {
                serde_json::to_string(p).unwrap()
            }
        }
    }
}

/*
 * Wrapper function for predictor
 * Based on the profile, delegates to the appropriate model
 */
pub fn predict(params : PredictorParams) -> Result<Prediction, String> {
    match params.profile {
        PredictionProfile::Standard => {
            standard_predict(StandardPredictorParams {
                launch: params.launch,

                burst_altitude: params.burst_altitude,
                ascent_rate: params.ascent_rate,
                descent_rate: params.descent_rate
            })
        },

        PredictionProfile::Float => {
            float_predict(FloatPredictionParams {
                launch: params.launch,

                duration: params.duration
            })
        },

        PredictionProfile::ValBal => {
            valbal_predict(FloatPredictionParams {
                launch: params.launch,

                duration: params.duration
            })
        }
    }
}

// TODO: Use Adams Bashforth Moulton for fancy, high quality integrals

fn standard_predict(params : StandardPredictorParams) -> Result<Prediction, String> {

    // TODO: implement checks to avoid infinite loops if ascent rate or descent rate is silly

    let mut current : Point = params.launch;

    // ascent
    let mut ascent : Vec<Point> = vec![];

    let ascent_velocity = Velocity {
        north: 0.0,
        east: 0.0,
        vertical: params.ascent_rate
    };

    while current.altitude < params.burst_altitude {
        let velocity = result_or_return!(velocity_at(&current)) + &ascent_velocity;

        current = current + &velocity;
        ascent.push(current.clone());
    }

    // burst
    let burst = current.clone();

    // descent
    let mut descent : Vec<Point> = vec![];

    let descent_velocity = Velocity {
        north: 0.0,
        east: 0.0,
        vertical: -params.descent_rate
    };
    while current.altitude > 0.0 {
        let velocity = result_or_return!(velocity_at(&current)) + &descent_velocity;

        current = current + &velocity;
        descent.push(current.clone());
    }

    Ok(Prediction::Standard(StandardPrediction {
        ascent, burst, descent
    }))
}

fn float_predict(params : FloatPredictionParams) -> Result<Prediction, String> {
    let mut current : Point = params.launch;
    let mut positions : Vec<Point> = vec![];

    let launch_time = current.clone().time;
    let end_time = launch_time + params.duration;

    while current.time < end_time {
        let velocity = result_or_return!(velocity_at(&current));

        current = current + &velocity;
        positions.push(current.clone());
    }

    Ok(Prediction::ValBal(FloatPrediction {
        positions
    }))
}

fn valbal_predict(params : FloatPredictionParams) -> Result<Prediction, String> {
    let mut current : Point = params.launch;
    let mut positions : Vec<Point> = vec![];

    let launch_time = current.clone().time;
    let end_time = launch_time + params.duration;

    while current.time < end_time {
        let velocity = result_or_return!(velocity_at(&current));

        current = current + &velocity;
        positions.push(current.clone());
    }

    Ok(Prediction::ValBal(FloatPrediction {
        positions
    }))
}


