use valbal::state::*;

/*
 * Figures out the instantaneous ascent rate, given a state
 * Does so by calculating lift, then modeling drag over the simulation_interval
 *
 * simulation_interval is in seconds
 */
pub fn calculate_ascent_rate(state : &ValBalState, simulation_interval : f32) -> f32 {
    let lift = get_lift(state);

    // model drag
    0.0
}

/*
 * Figures out the instantaneous lift, given a state
 */
fn get_lift(state : &ValBalState) -> f32 {

    lift_from_ballast(state.ballast_time, state.total_ballast_time, state.ballast_mass_rate) +
        lift_from_vent(state.vent_time, state.position.altitude, state.flight_time) +
        lift_from_temperature(state.temperature, state.outside_temperature)

}

/*
 * Calculates how much lift will change after venting for vent_time seconds
 *
 * vent_time is in seconds; this is the amount of time it vented for in the last iteration, not total
 * altitude is in meters
 * total_flight_time is in seconds
 */
fn lift_from_vent(vent_time : f32, altitude : f32, total_flight_time : f32) -> f32 {
    // physics
    0.0
}

/*
 * Calculates how much lift will change after ballasting for ballast_time seconds
 *
 * ballast_time is in seconds; this is the amount of time it ballasted for in the last iteration, not total
 * total_ballast_time is in seconds
 * ballast_mass_rate is in kg/s
 */
fn lift_from_ballast(ballast_time : f32, total_ballast_time : f32, ballast_mass_rate : f32) -> f32 {
    let delta_mass = ballast_time * ballast_mass_rate;

    // math to turn that into lift
    0.0
}

/*
 * Calculates the amount of lift produced by the temperature differential
 *
 * temperature is in degrees C; note that this is the temperature inside the envelope
 * outside_temperature is in degrees C
 */
fn lift_from_temperature(temperature : f32, outside_temperature : f32) -> f32 {
    // ideal gas laws or something
    0.0
}

/*
 * Figures out what the new internal temperature will be after exposed to an outside temperature for simulation_interval seconds
 * Assuming simulation_interval is short enough for behavior to be linear
 *
 * temperature is in degrees C
 * outside_temperature is in degrees C
 * simulation_interval is in seconds
 */
pub fn calculate_temperature(temperature : f32, outside_temperature : f32, simulation_interval : f32) -> f32 {
    // solve a differential equation or something
    0.0
}