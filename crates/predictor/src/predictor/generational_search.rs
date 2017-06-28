use std::mem;
use predictor::point::*;

struct Node {
    location : Point,
    previous : Link
}

enum Link {
    Empty,
    More(Box<Node>),
}

struct GenerationalPQueue {

}

impl Node {

    pub fn unravel(&mut self) -> Vec<Point> {
        let mut result : Vec<Point> = vec![];

        let mut cur_link = mem::replace(&mut self.previous, Link::Empty);
        result.push(self.location.clone());

        while let Link::More(mut boxed_node) = cur_link {
            cur_link = mem::replace(&mut boxed_node.previous, Link::Empty);
            result.push(boxed_node.location);
        }

        result
    }

    pub fn neighbors(&self) -> Vec<Node> {
        vec![]
    }

    pub fn from_point(point : Point) -> Node {
        Node {
            location : point,
            previous : Link::Empty
        }
    }
}

impl GenerationalPQueue {
    pub fn new() -> GenerationalPQueue {
        GenerationalPQueue {

        }
    }

    pub fn enqueue(&mut self, node : &Node) {

    }
}

pub fn search(start : Point) -> Vec<Point> {

    let mut best_yet = Node::from_point(start);

    {
        let mut queue = GenerationalPQueue::new();
        queue.enqueue(&best_yet);
    }

    best_yet.unravel()
}
