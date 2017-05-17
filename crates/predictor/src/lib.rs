#[macro_use]
extern crate helix;

#[macro_use]
extern crate lazy_static;

extern crate chrono;


pub mod predictor;

ruby! {
    class Predictor {
        def run(path: String) {
            predictor::GribReader::new(path);
        }
    }
}
