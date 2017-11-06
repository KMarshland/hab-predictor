use valbal::controller::*;
use valbal::state::*;

/*
 * This is the traditional ValBal controller that has been used since time immemorial
 * Copied from https://github.com/stanford-ssi/balloons-VALBAL/blob/a1d78e595a849edeffffec9c8a2328c5565d00a3/src/Controller.cpp
 */

const QUEUE_APPEND_THRESHOLD : f32 = 1.0;
const INCENTIVE_NOISE : f32 = 0.0;

pub struct PIDController {
    valve_setpoint : f32,
    valve_velocity_constant : f32,
    valve_altitude_difference_constant : f32,
    valve_last_action_constant : f32,

    ballast_setpoint : f32,
    ballast_velocity_constant : f32,
    ballast_altitude_difference_constant : f32,
    ballast_last_action_constant : f32,

    last_vent_altitude: f32,
    last_ballast_drop_altitude: f32,

    first_ballast_dropped: bool,
    re_arm_constant: f32,
    ballast_arm_altitude: f32,
    incentive_threshold: f32,

    ballast_altitude_last_default: f32,
    ballast_altitude_last_filler: f32,

    valve_duration: f32, // seconds
    ballast_duration: f32, // seconds

    valve_queue: f32, // seconds
    ballast_queue: f32 // seconds
}

impl ValBalController for PIDController {

    /*
     * Simulates the controller for `seconds` seconds
     * Returns the actions: how long it vented and how long it ballasted
     */
    fn run_iteration(&mut self, state : &ValBalState, seconds : f32) -> ControllerActionSet {

        let cycles_per_second = 1000.0 / 50.0;
        let seconds_per_cycle = 1.0 / cycles_per_second;
        let cycles = (seconds/cycles_per_second) as i32;

        let mut vent_time : f32 = 0.0;
        let mut ballast_time : f32 = 0.0;

        // run as many cycles of the algorithm as it takes to simulate for `seconds` seconds
        for _cycle in 0..cycles {

            // recalculate properties
            self.update_re_arm_constant();
            self.correct_altitude_since_last_vent(state);
            self.correct_altitude_since_last_dropped(state);

            // calculate incentives
            let valve_incentive = self.valve_incentive(state);
            let ballast_incentive = self.ballast_incentive(state);

            // enqueue actions
            if valve_incentive >= (1.0 + INCENTIVE_NOISE) && self.valve_queue <= QUEUE_APPEND_THRESHOLD {
                self.queue_vent();
            }

            if ballast_incentive >= (1.0 + INCENTIVE_NOISE) && self.ballast_queue <= QUEUE_APPEND_THRESHOLD {
                self.queue_ballast();
            }

            // convert queues to real amounts of time
            let cycle_vent_time = min!(self.valve_queue, seconds_per_cycle);
            if cycle_vent_time > 0.0 {
                vent_time += cycle_vent_time;
                self.valve_queue -= cycle_vent_time;
                self.last_vent_altitude = state.position.altitude;
            }

            let cycle_ballast_time = min!(self.ballast_queue, seconds_per_cycle);
            if cycle_ballast_time > 0.0 {
                ballast_time += cycle_ballast_time;
                self.ballast_queue -= cycle_ballast_time;
                self.last_ballast_drop_altitude = state.position.altitude;
            }
        }

        // return what you did
        ControllerActionSet {
            ballast_time,
            vent_time,

            duration: seconds
        }
    }

}

impl PIDController {

    /*
     * Creates a PID Controller with default parameters
     * Note that these can be updated partway through a simulation, eg by simulating RB
     * Values taken from https://github.com/stanford-ssi/balloons-VALBAL/blob/a1d78e595a849edeffffec9c8a2328c5565d00a3/src/Config.h
     */
    fn create_default() -> Self {
        PIDController {
            valve_setpoint: 14500.0,
            valve_velocity_constant: 1.0,
            valve_altitude_difference_constant: 1.0 / 1500.0,
            valve_last_action_constant : 1.0 / 1500.0,

            ballast_setpoint : 13500.0,
            ballast_velocity_constant : 1.0,
            ballast_altitude_difference_constant : 1.0 / 1500.0,
            ballast_last_action_constant : 1.0 / 1500.0,

            last_vent_altitude: 0.0,
            last_ballast_drop_altitude: -90000.0,

            first_ballast_dropped: false,
            re_arm_constant: 0.0,
            ballast_arm_altitude: 13250.0,
            incentive_threshold: 0.75,

            ballast_altitude_last_default: -90000.0,
            ballast_altitude_last_filler: 14000.0,

            valve_duration: 20.0,
            ballast_duration: 15.0,

            valve_queue: 0.0,
            ballast_queue: 0.0
        }
    }

    /*
     * Enqueues a vent action
     */
    fn queue_vent(&mut self) {
        self.valve_queue += self.valve_duration;
    }

    /*
     * Enqueues a ballast action
     */
    fn queue_ballast(&mut self) {
        self.ballast_queue += self.ballast_duration;
    }

    fn update_re_arm_constant(&mut self) {
        self.re_arm_constant = self.incentive_threshold / (self.ballast_altitude_difference_constant + self.ballast_last_action_constant);
    }

    /*
     * Corrects altitude since last ballast dropped, accounting for re-arming
     */
    fn correct_altitude_since_last_dropped(&mut self, state : &ValBalState) {
        let mut altitude_since_last_drop_corrected = self.last_ballast_drop_altitude;

        if !self.first_ballast_dropped && state.position.altitude >= self.ballast_arm_altitude &&
            self.last_ballast_drop_altitude == self.ballast_altitude_last_default
            {
                altitude_since_last_drop_corrected = self.ballast_altitude_last_filler;
                self.first_ballast_dropped = true;
            }

        if self.first_ballast_dropped {
            altitude_since_last_drop_corrected = max!(altitude_since_last_drop_corrected, state.position.altitude - self.re_arm_constant);
        }

        self.last_ballast_drop_altitude = altitude_since_last_drop_corrected;
    }

    /*
     * Corrects altitude since last venting, accounting for re-arming
     */
    fn correct_altitude_since_last_vent(&mut self, state : &ValBalState) {
        self.last_vent_altitude = min!(self.last_vent_altitude, state.position.altitude + self.re_arm_constant);
    }

    /*
     * Calculates the valve incentive, given the current controller state and current balloon state
     */
    fn valve_incentive(&self, state : &ValBalState) -> f32 {
        let proportional_term = self.valve_velocity_constant * state.ascent_rate;

        let integral_term = self.valve_altitude_difference_constant *
            (state.position.altitude - self.valve_setpoint);

        let derivative_term = self.valve_last_action_constant *
            (state.position.altitude - self.last_vent_altitude);

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
            (self.last_ballast_drop_altitude - state.position.altitude);

        return proportional_term + integral_term + derivative_term;
    }
}
