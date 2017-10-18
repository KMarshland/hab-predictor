use valbal::controller::*;
use valbal::state::*;

/*
 * This is the traditional ValBal controller that has been used since time immemorial
 * TODO: implement it
 */

pub struct PIDController {

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
