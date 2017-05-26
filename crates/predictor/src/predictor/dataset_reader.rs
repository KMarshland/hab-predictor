use std::sync::Mutex;
use std::fs;
use std::env;
use std::fs::DirEntry;
use chrono::prelude::*;
use predictor::point::*;
use predictor::grib_reader::*;

struct UninitializedDataSetReader {
    dataset_directory: String
}

struct DataSetReader {
    grib_readers: Vec<Box<GribReader>>
}

impl UninitializedDataSetReader {

    fn initialize(&mut self) -> DataSetReader {
        DataSetReader {
            grib_readers: {
                let start_time = UTC::now();

                let folders = fs::read_dir(self.dataset_directory.as_str()).unwrap();

                // figure out which data directory to read from
                let mut best_date = 0;
                let mut best_dir : Option<DirEntry> = None;

                for path in folders {
                    let dir = path.unwrap();

                    let file_name = dir.file_name();
                    let name = file_name.to_str().unwrap();

                    let date_num = name.parse::<i32>();

                    match date_num {
                        Ok(val) => {
                            if val > best_date {
                                best_date = val;
                                best_dir = Some(dir);
                            }
                        }
                        Err(_) => println!("Warning: junk file/folder in lib/data: {}", name),
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

                let mut readers : Vec<Box<GribReader>> = vec![];
                for file in bucket {
                    let path = file.path();

                    let extension = path.extension().unwrap().to_str().unwrap();

                    if extension == "grb2" {
                        println!("{}, extension {}", path.display(), extension);

                        // TODO: multithread this asshole
                        readers.push(Box::new(GribReader::new(path.to_str().unwrap().to_string())));
                    }
                }

                // TODO: enforce reader sort order

                println!("Readers initialized: {}ms", UTC::now().signed_duration_since(start_time).num_milliseconds());

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
        // TODO: Make it check the cache here

        let reader = self.get_reader(point);

        reader.velocity_at(point)
    }

    fn get_reader(&mut self, point: &Point) -> &Box<GribReader> {
        // TODO: implement a binary search tree or alternative fast lookup

        let mut best_reader = &self.grib_readers[0];

        for i in 1..self.grib_readers.len() {
            let reader = &self.grib_readers[i];
            let abs_seconds = reader.time.signed_duration_since(point.time).num_seconds().abs();
            let best_seconds = best_reader.time.signed_duration_since(point.time).num_seconds().abs();

            if abs_seconds < best_seconds {
                best_reader = reader;
            }
        }

        best_reader
    }
}

lazy_static! {
    static ref READER : Mutex<DataSetReader> = Mutex::new(DataSetReader::new(
        [env::var("RAILS_ROOT").expect("RAILS_ROOT environment variable not found"), "/lib/data".to_string()].concat()
    ));
}

pub fn velocity_at(point: &Point) -> Velocity {
    READER.lock().unwrap().velocity_at(&point)
}
