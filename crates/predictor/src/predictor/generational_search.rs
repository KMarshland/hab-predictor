use std::mem;
use std::cmp;
use std::collections::{BinaryHeap, VecDeque};
use chrono::prelude::*;
use chrono::Duration;
use predictor::point::*;

/*
 * Struct representing a single element in the queue
 * TODO: make this generic
 */
struct Node {
    location : Point,
    previous : Link,
    generation : usize,
    cost: f32
}

/*
 * Basis for node traversal
 */
enum Link {
    Empty,
    More(Box<Node>),
}

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
    pub fn unravel(&mut self) -> Vec<Point> {
        let mut result : VecDeque<Point> = VecDeque::new();

        let mut cur_link = mem::replace(&mut self.previous, Link::Empty);
        result.push_front(self.location.clone());

        while let Link::More(mut boxed_node) = cur_link {
            cur_link = mem::replace(&mut boxed_node.previous, Link::Empty);
            result.push_front(boxed_node.location);
        }

        let mut unreversed : Vec<Point> = vec![];

        while let Some(node) = result.pop_front() {
            unreversed.push(node)
        }

        unreversed
    }

    /*
     * TODO
     */
    pub fn neighbors(&self) -> Vec<Node> {
        vec![]
    }

    pub fn from_point(point : Point) -> Node {
        Node {
            location : point,
            previous : Link::Empty,
            generation: 0,
            cost: 0.0
        }
    }
}

/*
 * Make node able to be compared ordinally and thus stored in a priority queue
 */
impl cmp::Ord for Node {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        // comparing two floats so if this panics so will I
        self.cost.partial_cmp(&other.cost).unwrap()
    }
}

impl cmp::PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl cmp::PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
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
    pub fn new() -> GenerationalPQueue {
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
        for generation in self.costs.len()..(node.generation + 1) {
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
            let cost = self.generations[generation].peek().unwrap().cost + self.costs[generation];

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
pub fn search(start : Point, timeout : u32) -> Result<Vec<Point>, String> {

    let start_time = Local::now();

    let mut best_yet : Option<Node> = None;
    let mut best_score = 0.0;

    let mut queue = GenerationalPQueue::new();
    queue.enqueue(Node::from_point(start));

    // remember what the next generation is
    let mut next_gen = 0;

    // a counter that tells you how long it's been since you moved to the next generation
    let mut stagnation = 0;

    while let Option::Some(mut node) = queue.dequeue() {

        // check timeout
        if Local::now() > (start_time + Duration::seconds(timeout as i64)) {
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
        let mut children = node.neighbors();

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
        Some(mut node) => {
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
 * TODO: make this a lambda in params
 */
fn score(node : &Node) -> f32 {
    node.location.longitude
}
