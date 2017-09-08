extern crate chrono;
extern crate reqwest;

pub mod preprocessor;
use preprocessor::download::*;

fn main() {
    download().unwrap();
}