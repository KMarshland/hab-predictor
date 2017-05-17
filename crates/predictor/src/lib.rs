#[macro_use]
extern crate helix;
pub mod predictor;

ruby! {
    class Predictor {
        def run(path: String) {
            predictor::GribReader::new(path);
        }
    }
}
