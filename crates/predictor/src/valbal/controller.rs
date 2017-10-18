use valbal::state::*;

pub struct ControllerActionSet {
    pub ballast_time : f32, // seconds for which it dropped ballast in this action set
    pub vent_time : f32, // seconds for which it vented in this action set

    pub duration : f32 // seconds over which these actions are to be taken
}

pub trait ValBalController {

    /*
     * Simulates the controller for `seconds` seconds
     * Returns the actions: how long it vented and how long it ballasted
     * The controller may mutate itself to track any parameters it wishes
     */
    fn run_iteration(&mut self, state : &ValBalState, seconds : f32) -> ControllerActionSet;

    /*
     * Returns the new state after running an iteration of the controller
     */
    fn simulate_step(&mut self, state : ValBalState, seconds : f32) -> ValBalState {
        let actions = self.run_iteration(&state, seconds);
        state + &actions
    }

}
