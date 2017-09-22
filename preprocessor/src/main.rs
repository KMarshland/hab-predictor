extern crate chrono;
extern crate reqwest;

use chrono::prelude::*;

pub mod preprocessor;
use preprocessor::download::*;

fn main() {

    download().unwrap();

}
