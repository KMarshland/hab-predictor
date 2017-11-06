use valbal::controller::*;
use valbal::state::*;

/*
 * This is the traditional ValBal controller that has been used since time immemorial
 * Copied from https://github.com/stanford-ssi/balloons-VALBAL/blob/a1d78e595a849edeffffec9c8a2328c5565d00a3/src/Controller.cpp
 */

pub struct PIDController {
    valve_setpoint : f32,
    valve_velocity_constant : f32,
    valve_altitude_difference_constant : f32,
    valve_last_action_constant : f32,

    ballast_setpoint : f32,
    ballast_velocity_constant : f32,
    ballast_altitude_difference_constant : f32,
    ballast_last_action_constant : f32,

    altitude_since_last_vent_corrected: f32,
    altitude_since_last_ballast_dropped: f32,

}

impl ValBalController for PIDController {

    /*
     * Simulates the controller for `seconds` seconds
     * Returns the actions: how long it vented and how long it ballasted
     */
    fn run_iteration(&mut self, state : &ValBalState, seconds : f32) -> ControllerActionSet {
        ControllerActionSet {
            ballast_time: 0.0,
            vent_time: 0.0,

            duration: seconds
        }
    }

}

impl PIDController {

    fn valve_incentive(&self, state : &ValBalState) -> f32 {
        let proportional_term = self.valve_velocity_constant * state.ascent_rate;

        let integral_term = self.valve_altitude_difference_constant *
            (state.position.altitude - self.valve_setpoint);

        let derivative_term = self.valve_last_action_constant *
            (state.position.altitude - self.altitude_since_last_vent_corrected);

        return proportional_term + integral_term + derivative_term;
    }

    fn ballast_incentive(&self, state : &ValBalState) -> f32 {
        let proportional_term = self.ballast_velocity_constant * -1.0 * state.ascent_rate;

        let integral_term     = self.ballast_altitude_difference_constant *
            (self.ballast_setpoint - state.position.altitude);

        let derivative_term   = self.ballast_last_action_constant *
            (self.altitude_since_last_ballast_dropped - state.position.altitude);

        return proportional_term + integral_term + derivative_term;
    }
}
