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

    altitude_since_last_vent: f32,
    altitude_since_last_ballast_dropped: f32,

    first_ballast_dropped: bool,
    re_arm_constant: f32,
    ballast_arm_altitude: f32,
    incentive_threshold: f32,

    ballast_altitude_default: f32,
    ballast_altitude_last_filler: f32
}

impl ValBalController for PIDController {

    /*
     * Simulates the controller for `seconds` seconds
     * Returns the actions: how long it vented and how long it ballasted
     */
    fn run_iteration(&mut self, state : &ValBalState, seconds : f32) -> ControllerActionSet {

        self.update_re_arm_constant();
        self.correct_altitude_since_last_dropped(state);
        self.correct_altitude_since_last_vent(state);

        let valve_incentive = self.valve_incentive(state);
        let ballast_incentive = self.ballast_incentive(state);

        ControllerActionSet {
            ballast_time: 0.0,
            vent_time: 0.0,

            duration: seconds
        }
    }

}

impl PIDController {

//    fn create_default() -> Self {
//        PIDController {
//
//        }
//    }

    fn update_re_arm_constant(&mut self) {
        self.re_arm_constant = self.incentive_threshold / (self.ballast_altitude_difference_constant + self.ballast_last_action_constant);
    }

    /*
     * Corrects altitude since last ballast dropped, accounting for re-arming
     */
    fn correct_altitude_since_last_dropped(&mut self, state : &ValBalState) {
        let mut altitude_since_last_drop_corrected = self.altitude_since_last_ballast_dropped;

        if !self.first_ballast_dropped && state.position.altitude >= self.ballast_arm_altitude &&
            self.altitude_since_last_ballast_dropped == self.ballast_altitude_default
            {
                altitude_since_last_drop_corrected = self.ballast_altitude_last_filler;
                self.first_ballast_dropped = true;
            }

        if self.first_ballast_dropped {
            altitude_since_last_drop_corrected = max!(altitude_since_last_drop_corrected, state.position.altitude - self.re_arm_constant);
        }

        self.altitude_since_last_ballast_dropped = altitude_since_last_drop_corrected;
    }

    /*
     * Corrects altitude since last venting, accounting for re-arming
     */
    fn correct_altitude_since_last_vent(&mut self, state : &ValBalState) {
        self.altitude_since_last_vent = min!(self.altitude_since_last_vent, state.position.altitude + self.re_arm_constant);
    }

    /*
     * Calculates the valve incentive, given the current controller state and current balloon state
     */
    fn valve_incentive(&self, state : &ValBalState) -> f32 {
        let proportional_term = self.valve_velocity_constant * state.ascent_rate;

        let integral_term = self.valve_altitude_difference_constant *
            (state.position.altitude - self.valve_setpoint);

        let derivative_term = self.valve_last_action_constant *
            (state.position.altitude - self.altitude_since_last_vent);

        return proportional_term + integral_term + derivative_term;
    }

    /*
     * Calculates the ballast incentive, given the current controller state and current balloon state
     */
    fn ballast_incentive(&self, state : &ValBalState) -> f32 {
        let proportional_term = self.ballast_velocity_constant * -1.0 * state.ascent_rate;

        let integral_term     = self.ballast_altitude_difference_constant *
            (self.ballast_setpoint - state.position.altitude);

        let derivative_term   = self.ballast_last_action_constant *
            (self.altitude_since_last_ballast_dropped - state.position.altitude);

        return proportional_term + integral_term + derivative_term;
    }
}
