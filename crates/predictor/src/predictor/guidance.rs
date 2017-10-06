use libc;
use std::mem;
use std::collections::BinaryHeap;
use chrono::prelude::*;
use chrono::Duration;
use serde_json;

use predictor::point::*;
use predictor::predictor::*;
use predictor::guidance_node::*;

const DEFAULT_STAGNATION_COST : f32 = 0.1;
const STAGNATION_MULTIPLIER : f32 = 0.01;
pub const HEURISTIC_WEIGHT : f32 = 30.0;
pub const MOVEMENT_WEIGHT : f32 = 0.1;

pub struct GuidanceParams {
    pub launch : Point,

    pub timeout : f32, // seconds
    pub duration : Duration, // prevents it from circumnavigating indefinitely

    pub time_increment : Duration,

    pub altitude_variance : u32,
    pub altitude_increment : u32,

    pub compare_with_naive : bool,
    pub guidance_type: GuidanceType
}

pub enum GuidanceType {
    Distance,
    Destination(Point)
}

#[derive(Serialize)]
pub struct Guidance {
    metadata: GuidanceMetadata,
    positions: Vec<Point>,
    naive: Option<Vec<Point>>
}

#[derive(Serialize)]
struct GuidanceMetadata {
    nodes_checked : usize,
    generation: usize,
    max_generation_reached : usize
}

impl Guidance {
    pub fn serialize(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

pub fn guidance(params : GuidanceParams) -> Result<Guidance, String> {

    let score = score_for(&params);

    let mut result = {

        result_or_return!(search(&params, score))
    };

    let naive = match (&params).compare_with_naive {
        true => {
            let prediction = predict(PredictorParams {
                launch: (&params).launch.clone(),
                profile: PredictionProfile::ValBal,

                burst_altitude: 0.0,
                ascent_rate: 0.0,
                descent_rate: 0.0,

                duration: {
                    let first = match result.positions.first() {
                        Some(point) => point,
                        None => {
                            return Err(String::from("No data in naive prediction"));
                        }
                    };

                    let last = match result.positions.last() {
                        Some(point) => point,
                        None => {
                            return Err(String::from("No data in naive prediction"));
                        }
                    };

                    last.time.signed_duration_since(first.time)
                }
            });

            let naive_positions = match result_or_return!(prediction) {
                Prediction::ValBal(prediction) => {
                    prediction.positions
                },
                _ => {
                    panic!("Yikes (yeah, this shouldn't happen)");
                }
            };

            Some(naive_positions)
        },
        false => {
            None
        }
    };

    mem::replace(&mut result.naive, naive);

    Ok(result)
}

fn score_for(params : &GuidanceParams) -> Box<Fn(&Node) -> f32> {
    match &params.guidance_type {
        &GuidanceType::Destination(ref given_destination) => {
            let destination = given_destination.clone();

            let score = move |node : &Node| {
                -(node.location.longitude - destination.longitude).powi(2) - (node.location.latitude - destination.latitude).powi(2)
            };

            Box::new(score)
        },
        &GuidanceType::Distance => {
            let score = |node : &Node| {
                if node.location.longitude < -140.0 {
                    return node.location.longitude + 180.0 + 360.0
                }

                node.location.longitude + 180.0
            };

            Box::new(score)
        }
    }
}

// TODO: design a real data structure for this
struct GenerationalPQueue {
    costs : Vec<f32>,

    generations : Vec<BinaryHeap<*mut Node>>
}

/*
 * Data structure for that holds generations of priority queues
 * Within generations, costs are constant
 * However, the costs of generations as a whole can change frequently
 * This structure allows those costs to change with minimal overhead
 *
 * TODO: write tests
 */
impl GenerationalPQueue {
    pub fn new() -> Self {
        GenerationalPQueue {
            costs: vec![],
            generations: vec![]
        }
    }

    /*
     * Adds a node to the queue
     */
    pub fn enqueue(&mut self, node : *mut Node) {
        let generation = unsafe_dereference!(node).generation;

        self.allocate_at_least(generation);

        self.generations[generation].push(node);
    }

    /*
     * Pops a node from the queue
     */
    pub fn dequeue(&mut self) -> Option<*mut Node> {
        let mut best_generation : i32 = -1;
        let mut best_cost = 0.0;

        for generation in 0..self.generations.len() {
            if self.generations[generation].is_empty() {
                continue;
            }


            // unwrap will not panic: we already checked for emptiness
            let node_ptr = self.generations[generation].peek().unwrap().clone();

            let cost = unsafe_dereference!(node_ptr).cost() + self.costs[generation];

            if best_generation == -1 || cost < best_cost {
                best_cost = cost;
                best_generation = generation as i32
            }
        }

        // no data in any of the pqueues
        if best_generation == -1 {
            return None
        }

        self.generations[best_generation as usize].pop()
    }

    /*
     * Sets the generational cost
     * Does not affect underlying pqueues
     */
    pub fn set_cost(&mut self, generation : usize, cost : f32) {
        self.allocate_at_least(generation);

        self.costs[generation] = cost
    }

    /*
     * Makes sure the underlying queues and arrays can support at least `generations` generations
     */
    fn allocate_at_least(&mut self, generations : usize) {
        for _ in self.costs.len()..(generations + 1) {
            self.costs.push(DEFAULT_STAGNATION_COST);
            self.generations.push(BinaryHeap::new())
        }
    }
}

/*
 * Does greedy search, starting from the start point and going for timeout seconds
 */
fn search(params : &GuidanceParams, score: Box<Fn(&Node) -> f32>) -> Result<Guidance, String> {

    let mut free_at_end : Vec<*mut Node> = Vec::new();
    let end_time = Local::now() + Duration::seconds(params.timeout as i64);

    let mut best_yet : Option<*mut Node> = None;
    let mut best_score = 0.0;

    let start = Node::from_point(params.launch.clone());
    free_at_end.push(start);

    let mut queue = GenerationalPQueue::new();
    queue.enqueue(start);

    // remember what the next generation is
    let mut next_gen = 0;

    // a counter that tells you how long it's been since you moved to the next generation
    let mut stagnation = 0;

    // only used for debugging
    let mut checked = 0;
    let mut max_generation = 0;


    while let Option::Some(node_ptr) = queue.dequeue() {

        // check timeout
        if Local::now() > end_time {
            break;
        }

        let node = unsafe_dereference!(node_ptr);

        if node.generation > max_generation {
            max_generation = node.generation
        }

        // recalculate generational cost

        if (node.generation + 1) > next_gen {
            // you're continuing to make progress along this path. Yay!
            next_gen = node.generation + 1;
            stagnation = 0;

        } else {
            // you're stagnating faster than a mosquito bucket
            stagnation += 1;
        }

        if next_gen == 0 || next_gen == 1 {
            // you can't very well be stagnating on the first generation
            queue.set_cost(next_gen, 0.0);
        } else {
            // as stagnation increases, make the next generation look more appealing
            queue.set_cost(next_gen, (-STAGNATION_MULTIPLIER * (stagnation as f32)) + DEFAULT_STAGNATION_COST);
        }

        // enqueue children
        let mut children = result_or_return!(node.neighbors(node_ptr, &params));

        while !children.is_empty() {
            // TODO: make a preliminary filter on children's cost

            let child = children.pop().unwrap();
            free_at_end.push(child);

            queue.enqueue(child);
        }

        // see if you're doing better than before
        // Note: this must come at the end, as it potentially takes ownership of node
        match best_yet {
            Some(_) => {
                let new_score = score(&node);

                if new_score > best_score {
                    best_yet = Some(node_ptr);
                    best_score = new_score;
                }
            },
            None => {
                best_yet = Some(node)
            }
        }

        checked += 1;
    }

    match best_yet {
        Some(node) => {
            let final_generation = unsafe_dereference!(node).generation;
            let positions = unsafe_dereference!(node).unravel();

            while !free_at_end.is_empty() {
                unsafe {
                    libc::free(free_at_end.pop().unwrap() as *mut libc::c_void);
                }
            }

            Ok(Guidance{
                metadata: GuidanceMetadata {
                    generation: final_generation,
                    max_generation_reached: max_generation,
                    nodes_checked: checked
                },
                positions: positions,
                naive: None
            })
        },
        None => {
            Err(String::from("Best node not found (this error should never occur)"))
        }
    }
}