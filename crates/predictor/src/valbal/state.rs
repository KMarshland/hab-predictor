use predictor::point::*;

pub struct ValBalState {
    pub position : Point,

    pub outside_temperature : f32,
    pub temperature : f32,

    pub ballast_mass_rate : f32, // kg/s

    pub total_ballast_time : f32, // s
    pub total_vent_time : f32, // s
    pub ballast_time : f32, // s; amount ballasted in last iteration
    pub vent_time : f32, // s; amount vented in last iteration

    pub flight_time : f32 // s
}

