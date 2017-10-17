use std::collections::BinaryHeap;

use navigation::navigation::*;
use navigation::navigation_node::*;

// TODO: design a real data structure for this
pub struct GenerationalPQueue {
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
