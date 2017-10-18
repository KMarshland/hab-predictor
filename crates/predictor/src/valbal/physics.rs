use valbal::state::*;


pub fn calculate_ascent_rate(state : ValBalState) -> f32 {
    let lift = get_lift(state);

    // model drag
    0.0
}

fn get_lift(state : ValBalState) -> f32 {

    lift_from_ballast(state.ballast_time, state.total_ballast_time, state.ballast_mass_rate) +
        lift_from_vent(state.vent_time, state.position.altitude, state.flight_time) +
        lift_from_temperature(state.temperature, state.outside_temperature)

}

fn lift_from_vent(vent_time : f32, altitude : f32, total_flight_time : f32) -> f32 {
    // physics
    0.0
}

fn lift_from_ballast(ballast_time : f32, total_ballast_time : f32, ballast_mass_rate : f32) -> f32 {
    let delta_mass = ballast_time * ballast_mass_rate;

    // math to turn that into lift
    0.0
}

fn lift_from_temperature(temperature : f32, outside_temperature : f32) -> f32 {
    // ideal gas laws or something
    0.0
}

pub fn calculate_temperature(temperature : f32, outside_temperature : f32) -> f32 {
    // solve a differential equation or something
    0.0
}