use valbal::controller::*;
use valbal::state::*;

pub struct PIDController {

}

impl ValBalController for PIDController {

    fn simulate_step(&mut self, state : ValBalState, seconds : f32) -> ValBalState {
        // TODO
        state
    }

}
