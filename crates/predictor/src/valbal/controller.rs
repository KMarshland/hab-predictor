use valbal::state::*;

pub trait ValBalController {

    fn simulate_step(&mut self, state : ValBalState, seconds : f32) -> ValBalState;

}
