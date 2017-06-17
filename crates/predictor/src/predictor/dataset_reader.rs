use std::sync::Mutex;
use std::fs;
use std::env;
use std::fs::DirEntry;
use std::mem;
use std::cell::Cell;
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

    fn initialize(&mut self) -> Result<DataSetReader, String> {
        Ok(DataSetReader {
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
                        Err(_) => {
                            //                            println!("Warning: junk file/folder in lib/data: {}", name)
                        },
                    }
                }

                if best_date == 0 {
                    return Err(String::from("No data found"));
                }

                let mut bucket0 : Vec<DirEntry> = vec![];
                let mut bucket6 : Vec<DirEntry> = vec![];
                let mut bucket12 : Vec<DirEntry> = vec![];
                let mut bucket18 : Vec<DirEntry> = vec![];

                let files = fs::read_dir(best_dir.unwrap().path()).unwrap();
                for path in files {
                    let file = path.unwrap();
                    let undone_path = file.path();

                    // nested matching makes me want to die. We should refactor
                    match undone_path.extension() {
                        Some(ext) => {
                            match ext.to_str() {
                                Some(extension) => {
                                    if extension != "grb2" {
                                        continue
                                    }
                                }
                                None => {
                                    continue
                                }
                            }
                        }
                        None => {
                            continue
                        }
                    }

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
                        let reader = GribReader::new(path.to_str().unwrap().to_string());

                        readers.push(Box::new(reader));
                    }
                }

                // TODO: enforce reader sort order

                println!("Readers initialized: {}ms", UTC::now().signed_duration_since(start_time).num_milliseconds());

                readers
            }
        })
    }
}

impl DataSetReader {

    pub fn velocity_at(&mut self, point: &Point) -> Result<Velocity, String> {
        // TODO: Make it check the cache here

        let reader = self.get_reader(point);

        Ok(reader.velocity_at(point))
    }

    fn get_reader(&mut self, point: &Point) -> &Box<GribReader> {
        // TODO: implement a binary search tree or alternative fast lookup

        let readers = &self.grib_readers;

        let mut best_reader = &readers[0];

        for i in 1..self.grib_readers.len() {
            let reader = &readers[i];

            let abs_seconds = reader.time.signed_duration_since(point.time).num_seconds().abs();
            let best_seconds = best_reader.time.signed_duration_since(point.time).num_seconds().abs();

            if abs_seconds < best_seconds {
                best_reader = reader;
            }
        }

        best_reader
    }
}

struct WrappedDataSetReader {
    dataset_directory: String,
    reader: Option<Box<DataSetReader>>
}

impl WrappedDataSetReader {

    /*
     * Function to get the dataset reader
     * If none exists, it will try initializing one
     */
    pub fn get(&mut self) -> Result<Box<DataSetReader>, String> {
        let reader = mem::replace(&mut self.reader, None);

        match reader {
            Some(dataset_box) => {
                Ok(dataset_box)
            },

            None => {
                let mut uninitialized = UninitializedDataSetReader {
                    dataset_directory: self.dataset_directory.clone()
                };

                let initialized = uninitialized.initialize();

                match initialized {
                    Ok(initialized_reader) => {
                        let boxed = Box::new(initialized_reader);

                        self.reader = Some(boxed);

                        self.get()
                    },
                    Err(why) => {
                        return Err(why)
                    }
                }
            }
        }
    }

    pub fn new(dataset_directory : String) -> Self {
        WrappedDataSetReader {
            dataset_directory: dataset_directory,
            reader: None
        }
    }
}

lazy_static! {
    static ref READER : Mutex<WrappedDataSetReader> = Mutex::new(WrappedDataSetReader::new(
        [env::var("RAILS_ROOT").expect("RAILS_ROOT environment variable not found"), "/lib/data".to_string()].concat()
    ));
}

pub fn velocity_at(point: &Point) -> Velocity {
    let result = match READER.lock().unwrap().get() {
        Ok(mut reader) => {
            Ok(reader.velocity_at(&point))
        },
        Err(why) => {
            Err(why)
        }
    };

    match result {
        Ok(vel) => {
            vel.unwrap()
        },
        Err(why) => {
            panic!(why)
        }
    }
}
