use std::sync::Mutex;
use predictor::point::*;

struct UninitializedDataSetReader {
    dataset_directory: String
}

struct DataSetReader {
    dataset_directory: String
}

impl UninitializedDataSetReader {

    fn initialize(&mut self) -> DataSetReader {
        DataSetReader {
            dataset_directory: self.dataset_directory.clone()
        }
    }
}

impl DataSetReader {

    pub fn new(dataset_directory : String) -> DataSetReader {
        let mut reader = UninitializedDataSetReader {
            dataset_directory: dataset_directory
        };

        reader.initialize()
    }

    pub fn velocity_at(&mut self, point: &Point) -> Velocity {
        unimplemented!();

        // TODO: find the right grib reader, then call velocity at on it
        // Also, cache
    }
}

lazy_static! {
    static ref READER : Mutex<DataSetReader> =  Mutex::new(DataSetReader::new(
        "/Users/kaimarshland/Programming/ssi/prediction/lib/data".to_string()
    ));
}

pub fn velocity_at(point: &Point) -> Velocity {
    READER.lock().unwrap().velocity_at(&point)
}
