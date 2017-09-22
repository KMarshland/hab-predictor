extern crate chrono;
extern crate reqwest;

use chrono::prelude::*;

pub mod preprocessor;
use preprocessor::download::*;

fn main() {

//    let at = download().unwrap();

    // test preprocessor
    let at = chrono::Utc.ymd(2017, 9, 16).and_hms(0, 0, 0);

    preprocessor::preprocess(at).unwrap();

}
