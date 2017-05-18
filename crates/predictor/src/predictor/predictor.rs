use predictor::point::*;
use predictor::dataset_reader::velocity_at;

pub enum PredictionProfile {
    Standard,
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
    pub duration: f32, // minutes
}

/*
 * Only those parameters necessary to run a standard profile prediction
 */
#[allow(dead_code)]
struct StandardPredictorParams {
    launch: Point,

    // standard profile
    burst_altitude: f32, // meters
    ascent_rate: f32, // meters per second
    descent_rate: f32, // meters per second
}

/*
 * Only those parameters necessary to run a ValBal prediction
 */
#[allow(dead_code)]
struct ValBalPredictorParams {
    launch: Point,

    // valbal
    duration: f32, // minutes
}

/*
 * The result of a prediction
 */
pub struct Prediction {
    result: Vec<Point>
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

        PredictionProfile::ValBal => {
            valbal_predict(ValBalPredictorParams {
                launch: params.launch,

                duration: params.duration
            })
        }
    }
}

fn standard_predict(params : StandardPredictorParams) -> Result<Prediction, String> {

    let mut current : Point = params.launch;
    let mut positions : Vec<Point> = vec![];

    // ascent
    let ascent_velocity = Velocity {
        north: 0.0,
        east: 0.0,
        vertical: params.ascent_rate
    };

    while current.altitude < params.burst_altitude {
        let velocity = velocity_at(&current) + &ascent_velocity;

        current = current + &velocity;
        positions.push(current.clone());
    }

    // burst
    let burst = current.clone();

    // descent
    let descent_velocity = Velocity {
        north: 0.0,
        east: 0.0,
        vertical: params.descent_rate
    };
    while current.altitude > 0.0 {
        let velocity = velocity_at(&current) + &descent_velocity;

        current = current + &velocity;
        positions.push(current.clone());
    }

    Ok(Prediction {
        result: positions
    })
}

fn valbal_predict(params : ValBalPredictorParams) -> Result<Prediction, String> {
    Ok(Prediction {
        result: vec![]
    })
}


