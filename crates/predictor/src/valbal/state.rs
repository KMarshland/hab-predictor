use std::ops::Add;

use predictor::point::*;
use valbal::controller::*;

/*
 * Represents the current state of the physical ValBal system
 * Note that this is meant to cover physical properties only
 * Anything related to software (eg controller state) should live on the controller
 */
pub struct ValBalState {
    pub ascent_rate : f32, // m/s
    pub position : Point,

    pub outside_temperature : f32, // degrees C
    pub temperature : f32, // degrees C; temperature of the gas inside the balloon envelope

    pub ballast_mass_rate : f32, // kg/s

    pub total_ballast_time : f32, // s
    pub total_vent_time : f32, // s
    pub ballast_time : f32, // s; amount ballasted in last iteration
    pub vent_time : f32, // s; amount vented in last iteration

    pub flight_time : f32 // s
}

/*
 * Adding an action set updates the action-related values in the state
 * Note that this does not do any physics simulation; that is handled elsewhere
 */
impl<'a> Add<&'a ControllerActionSet> for ValBalState {
    type Output = ValBalState;

    fn add(self, actions: &'a ControllerActionSet) -> ValBalState {
        ValBalState {
            ascent_rate: self.ascent_rate,
            position: self.position,

            outside_temperature: self.outside_temperature,
            temperature: self.temperature,

            ballast_mass_rate: self.ballast_mass_rate,

            total_ballast_time : self.total_ballast_time + actions.ballast_time,
            total_vent_time : self.total_vent_time + actions.vent_time,
            ballast_time : actions.ballast_time,
            vent_time : actions.vent_time,

            flight_time: self.flight_time + actions.duration
        }
    }
}
