use std::mem;
use std::cmp;
use std::collections::{BinaryHeap, VecDeque};
use predictor::point::*;

struct Node {
    location : Point,
    previous : Link,
    generation : usize,
    cost: f32
}

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


impl GenerationalPQueue {
    pub fn new() -> GenerationalPQueue {
        GenerationalPQueue {
            costs: vec![],
            generations: vec![]
        }
    }

    pub fn enqueue(&mut self, node : Node) {

        // initialize costs and queues if we need
        for generation in self.costs.len()..(node.generation + 1) {
            self.costs.push(0.1);
            self.generations.push(BinaryHeap::new())
        }

        self.generations[node.generation].push(node);
    }

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

    pub fn set_cost(&mut self, generation : usize, cost : f32) {
        self.costs[generation] = cost
    }
}

pub fn search(start : Point) -> Vec<Point> {

    let mut best_yet : Option<Node> = None;

    let mut queue = GenerationalPQueue::new();
    queue.enqueue(Node::from_point(start));

    //        while let Option::Some(node) = queue.dequeue() {
    //
    //        }

    best_yet.unwrap().unravel()
}
