use std::cmp;
use std::mem;
use std::f32;
use libc;
use std::collections::VecDeque;

use predictor::point::*;
use predictor::predictor::*;
use navigation::navigation::*;

/*
 * Struct representing a single element in the queue
 * TODO: make this generic
 */
pub struct Node {
    pub location : Point,
    previous : Option<*const Node>,

    pub generation : usize,

    pub heuristic_cost: f32,
    movement_cost: f32
}

impl Node {

    /*
     * Destructively walks up this node, turning it into a vector (in order)
     */
    pub fn unravel(&self) -> Vec<Point> {
        let mut result : VecDeque<Point> = VecDeque::new();

        result.push_front(self.location.clone());
        let mut previous = self.previous.clone();

        let mut iterations = 0;

        loop {
            previous = match previous {
                Some(node_ptr) => {
                    // yikes
                    let node = unsafe  { // please don't SEGFAULT
                        let ref node = *node_ptr;
                        node
                    };

                    result.push_front(node.location.clone());

                    node.previous
                }
                None => {
                    break;
                }
            };

            if iterations > self.generation {
                panic!("Too many iterations in unraveling");
            }
            iterations += 1;
        }

        let mut unreversed : Vec<Point> = vec![];

        while let Some(node) = result.pop_front() {
            unreversed.push(node)
        }

        unreversed
    }

    /*
     * Gets the neighbors of this node by making a prediction
     */
    pub fn neighbors(&self, address : *mut Self, params : &NavigationParams) -> Result<Vec<*mut Self>, String> {
        // return blank if you're at the end of the time period
        if (self.generation as i64)*params.time_increment.num_seconds() > params.duration.num_seconds() {
            return Ok(vec![]);
        }

        let prediction = predict(PredictorParams {
            launch: self.location.clone(),
            profile: PredictionProfile::ValBal,

            burst_altitude: 0.0,
            ascent_rate: 0.0,
            descent_rate: 0.0,

            duration: params.time_increment
        });

        let point = match prediction {
            Ok(unwrapped) => {
                match unwrapped {
                    Prediction::ValBal(prediction) => {
                        let mut borrowed = prediction;
                        match borrowed.positions.pop() {
                            Some(point) => {
                                point
                            },
                            _ => {
                                return Err(String::from("No data in prediction"));
                            }
                        }
                    },
                    _ => {
                        panic!("Yikes (yeah, this shouldn't happen)");
                    }
                }
            },
            Err(why) => {
                return Err(why);
            }
        };

        let mut result : Vec<*mut Node> = Vec::new();

        for multiplier in (-(params.altitude_variance as i32))..((params.altitude_variance as i32) + 1) {
            let altitude = point.altitude + ((multiplier as f32) * (params.altitude_increment as f32));

            if altitude < 0.0 {
                // don't fly into the ground :skeleton:
                continue;
            }

            if altitude > 20_000.0 {
                // don't fly too high, Icarus
                continue;
            }

            let child = unsafe {
                let child_ptr : *mut Node = libc::malloc(mem::size_of::<Node>()) as *mut Node;
                if child_ptr.is_null() {
                    panic!("Failed to allocate a new node");
                }

                *child_ptr = Node {
                    location: Point {
                        time: point.time,
                        latitude: point.latitude,
                        longitude: point.longitude,
                        altitude
                    },

                    previous: Some(address),

                    generation: self.generation + 1,

                    heuristic_cost: {

                        let multiplier = HEURISTIC_WEIGHT / (params.time_increment.num_seconds() as f32);

                        let cost : f32 = match &params.navigation_type {
                            &NavigationType::Destination(ref destination) => {
                                point.distance_to(destination) * HEURISTIC_WEIGHT
                            },
                            &NavigationType::Distance => {
                                if self.location.longitude > 0.0 && point.longitude < 0.0 {
                                    (self.location.longitude - (point.longitude + 360.0)) * multiplier
                                } else if self.location.longitude < 0.0 && point.longitude > 0.0 {
                                    ((self.location.longitude + 360.0) - point.longitude) * multiplier
                                } else {
                                    (self.location.longitude - point.longitude) * multiplier
                                }
                            }
                        };

                        cost
                    },

                    movement_cost: {
                        self.movement_cost + ((self.location.altitude - altitude).abs().sqrt() * MOVEMENT_WEIGHT)
                    }
                };

                child_ptr
            };

            result.push(child);
        }

        Ok(result)
    }

    pub fn from_point(point : Point) -> *mut Self {

        let node_ptr = unsafe {
            let node_ptr: *mut Node = libc::malloc(mem::size_of::<Node>()) as *mut Node;
            if node_ptr.is_null() {
                panic!("Failed to allocate a new node");
            }

            node_ptr
        };

        unsafe {
            *node_ptr = Node {
                location: point,
                previous: None,

                generation: 0,

                heuristic_cost: f32::INFINITY,
                movement_cost: 0.0
            };
        }

        node_ptr
    }

    /*
     * Total cost for a node
     */
    pub fn cost(&self) -> f32 {
        self.heuristic_cost + self.movement_cost
    }
}

/*
 * Make node able to be compared ordinally and thus stored in a priority queue
 */
impl cmp::Ord for Node {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        // comparing two floats so if this panics so will I
        self.cost().partial_cmp(&other.cost()).unwrap()
    }
}

impl cmp::PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl cmp::PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.cost() == other.cost()
    }
}

impl cmp::Eq for Node { }
