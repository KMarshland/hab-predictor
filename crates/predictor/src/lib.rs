#[macro_use]
extern crate helix;
extern crate chrono;
pub mod predictor;

ruby! {
    class Predictor {
        def run(path: String) {
            predictor::GribReader::new(path);
        }
    }
}
