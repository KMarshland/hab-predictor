use std::cmp;
use std::collections::{BinaryHeap, VecDeque};
use chrono::prelude::*;
use chrono::Duration;
use serde_json;
use predictor::point::*;
use predictor::predictor::*;

pub struct GuidanceParams {
    pub launch : Point,

    pub timeout : f32,

    pub time_increment : f32, // minutes

    pub altitude_variance : u32,
    pub altitude_increment : u32
}

#[derive(Serialize)]
pub struct Guidance {
    positions: Vec<Point>
}

impl Guidance {
    pub fn serialize(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

pub fn guidance(params : GuidanceParams) -> Result<Guidance, String> {
    let positions = result_or_return!(search(params));

    Ok(Guidance {
        positions: positions
    })
}
/*
 * Struct representing a single element in the queue
 * TODO: make this generic
 */
struct Node {
    location : Point,
    previous : Option<*const Node>,

    generation : usize,

    heuristic_cost: f32,
    movement_cost: f32
}

/*
 * Basis for node traversal
 */
//enum Link {
//    Empty,
//    Another(*const Node),
//}

// TODO: design a real data structure for this
struct GenerationalPQueue {
    costs : Vec<f32>,

    // TODO: use fibonacci heaps for underlying implementation
    generations : Vec<BinaryHeap<Node>>
}

impl Node {

    /*
     * Destructively walks up this node, turning it into a vector (in order)
     */
    fn unravel(&self) -> Vec<Point> {
        let mut result : VecDeque<Point> = VecDeque::new();

        result.push_front(self.location.clone());
        let mut previous = self.previous.clone();

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
    fn neighbors(&self, params : &GuidanceParams) -> Result<Vec<Node>, String> {
        let prediction = predict(PredictorParams {
            launch: self.location.clone(),
            profile: PredictionProfile::ValBal,

            burst_altitude: 0.0,
            ascent_rate: 0.0,
            descent_rate: 0.0,

            duration: 60.0
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

        let mut result : Vec<Node> = Vec::new();

        for multiplier in (-(params.altitude_variance as i32))..((params.altitude_variance as i32) + 1) {
            let altitude = point.altitude + ((multiplier as f32) * (params.altitude_increment as f32));

            if altitude < 0.0 {
                // don't fly into the ground :skeleton:
                continue;
            }

            result.push(Node {
                location: Point {
                    time: point.time,
                    latitude: point.latitude,
                    longitude: point.longitude,
                    altitude: altitude
                },
                previous: Some(&*self), // this syntax makes me want to die but it gets a pointer to self

                generation: self.generation + 1,
                heuristic_cost: {
                    (self.location.longitude - point.longitude) / params.time_increment
                },
                movement_cost: {
                    // TODO: make this proportional to the square of the change without scaling it too weirdly
                    self.movement_cost + (self.location.altitude - altitude).abs()/1200000.0
                }
            })
        }

        Ok(result)
    }

    fn from_point(point : Point) -> Self {
        Node {
            location : point,
            previous : None,

            generation: 0,

            heuristic_cost: 0.0,
            movement_cost: 0.0
        }
    }

    /*
     * Total cost for a node
     */
    fn cost(&self) -> f32 {
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
    pub fn enqueue(&mut self, node : Node) {

        // initialize costs and queues if we need
        for _ in self.costs.len()..(node.generation + 1) {
            self.costs.push(0.1);
            self.generations.push(BinaryHeap::new())
        }

        self.generations[node.generation].push(node);
    }

    /*
     * Pops a node from the queue
     */
    pub fn dequeue(&mut self) -> Option<Node> {
        let mut best_generation : i32 = -1;
        let mut best_cost = 0.0;

        for generation in 0..self.generations.len() {
            if self.generations[generation].is_empty() {
                continue;
            }


            // unwrap will not panic: we already checked for emptiness
            let cost = self.generations[generation].peek().unwrap().cost() + self.costs[generation];

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
        self.costs[generation] = cost
    }
}

/*
 * Does greedy search, starting from the start point and going for timeout seconds
 */
fn search(params : GuidanceParams) -> Result<Vec<Point>, String> {

    let end_time = Local::now() + Duration::seconds(params.timeout as i64);

    let mut best_yet : Option<Node> = None;
    let mut best_score = 0.0;

    let mut queue = GenerationalPQueue::new();
    queue.enqueue(Node::from_point(params.launch.clone()));

    // remember what the next generation is
    let mut next_gen = 0;

    // a counter that tells you how long it's been since you moved to the next generation
    let mut stagnation = 0;

    while let Option::Some(node) = queue.dequeue() {

        // check timeout
        if Local::now() > end_time {
            break;
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
            queue.set_cost(next_gen, (-0.01 * (stagnation as f32)) + 0.1);
        }

        // enqueue children
        let mut children = result_or_return!(node.neighbors(&params));

        while !children.is_empty() {
            // TODO: make a preliminary filter on children's cost

            // unwrap should not panic: we're already checking for emptiness
            queue.enqueue(children.pop().unwrap())
        }

        // see if you're doing better than before
        // Note: this must come at the end, as it potentially takes ownership of node
        match best_yet {
            Some(_) => {
                let new_score = score(&node);

                if new_score > best_score {
                    best_yet = Some(node);
                    best_score = new_score;
                }
            },
            None => {
                best_yet = Some(node)
            }
        }
    }

    match best_yet {
        Some(node) => {
            Ok(node.unravel())
        },
        None => {
            Err(String::from("Best node not found (this error should never occur)"))
        }
    }
}

/*
 * Converts a node into a representation of how good it is
 * Higher is better
 */
fn score(node : &Node) -> f32 {
    node.location.longitude
}