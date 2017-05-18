use std::sync::Mutex;
use std::fs;
use std::fs::DirEntry;
use predictor::point::*;
use predictor::grib_reader::*;

struct UninitializedDataSetReader {
    dataset_directory: String
}

struct DataSetReader {
    grib_readers: Vec<GribReader>
}

impl UninitializedDataSetReader {

    fn initialize(&mut self) -> DataSetReader {
        DataSetReader {
            grib_readers: {
                let folders = fs::read_dir(self.dataset_directory.as_str()).unwrap();

                // figure out which data directory to read from
                let mut best_date = 0;
                let mut best_dir : Option<DirEntry> = None;

                for path in folders {
                    let dir = path.unwrap();

                    let file_name = dir.file_name();
                    let name = file_name.to_str().unwrap();

                    let date_num = name.parse::<i32>().unwrap();

                    if date_num > best_date {
                        best_date = date_num;
                        best_dir = Some(dir);
                    }
                }

                if best_date == 0 {
                    panic!("No data found")
                }

                let mut bucket0 : Vec<DirEntry> = vec![];
                let mut bucket6 : Vec<DirEntry> = vec![];
                let mut bucket12 : Vec<DirEntry> = vec![];
                let mut bucket18 : Vec<DirEntry> = vec![];

                let files = fs::read_dir(best_dir.unwrap().path()).unwrap();
                for path in files {
                    let file = path.unwrap();
                    let file_name = file.file_name();
                    let name = file_name.to_str().unwrap();

                    let parts = name.split("_").collect::<Vec<&str>>();
                    let bucket = parts[3];

                    match bucket.as_ref() {
                        "1800" => {
                            bucket18.push(file)
                        },
                        "1200" => {
                            bucket12.push(file)
                        },
                        "0600" => {
                            bucket6.push(file)
                        },
                        _ => {
                            bucket0.push(file)
                        }
                    }
                }

                let bucket : Vec<DirEntry> = {
                    if bucket18.len() > 0 {
                        bucket18
                    } else if bucket12.len() > 0 {
                        bucket12
                    } else if bucket6.len() > 0 {
                        bucket6
                    } else {
                        bucket0
                    }
                };

                let mut readers : Vec<GribReader> = vec![];
                for file in bucket {
                    println!("{}", file.path().display());

                    readers.push(GribReader::new(file.path().to_str().unwrap().to_string()));
                }

                readers
            }
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
        // TODO: find the right grib reader, then call velocity at on it
        // Also, cache

        Velocity {
            north: 1.0,
            east: 1.0,
            vertical: 1.0
        }
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
