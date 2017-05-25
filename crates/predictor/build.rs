extern crate gcc;

fn main() {
    gcc::Config::new()
        .file("src/predictor/wgrib.c")
        .include("src")
        .compile("libgrib.a");
}