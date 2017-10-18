use valbal::state::*;

pub trait ValBalController {

    fn simulate_step(state : ValBalState, seconds : f32) -> ValBalState;

}

pub enum ControllerType {
    PID
}