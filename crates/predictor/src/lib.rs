#[macro_use]
extern crate helix;

ruby! {
    class Predictor {
        def hello() {
            println!("Hello from predictor!");
        }
    }
}
