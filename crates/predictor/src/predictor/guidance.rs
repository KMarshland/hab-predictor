use serde_json;
use predictor::point::*;
use predictor::predictor::*;

pub struct GuidanceParams {
    pub launch : Point,

    pub performance : u32,
    pub timeout : u32,
    pub altitude_res : u32
}

#[derive(Serialize)]
pub struct Guidance {
    positions: Vec<Point>
}

impl Guidance {
    pub fn serialize(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

pub fn guidance(params : GuidanceParams) -> Result<Guidance, String> {
    Err(String::from("Yikes"))
}
