use predictor::point::Point;

pub enum PredictionProfile {
    Standard,
    ValBal
}

/*
 * All parameters that might be passed in to the prediction
 */
pub struct PredictorParams {
    launch: Point,

    profile: PredictionProfile,

    // standard profile
    burst_altitude: i32, // meters
    ascent_rate: i32, // meters per second
    descent_rate: i32, // meters per second

    // valbal
    duration: f32, // minutes
}

/*
 * Only those parameters necessary to run a standard profile prediction
 */
#[allow(dead_code)]
struct StandardPredictorParams {
    launch: Point,

    // standard profile
    burst_altitude: i32, // meters
    ascent_rate: i32, // meters per second
    descent_rate: i32, // meters per second
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
    Ok(Prediction {
        result: vec![]
    })
}

fn valbal_predict(params : ValBalPredictorParams) -> Result<Prediction, String> {
    Ok(Prediction {
        result: vec![]
    })
}


