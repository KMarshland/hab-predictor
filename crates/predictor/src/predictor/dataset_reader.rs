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

    fn initialize(&mut self) -> Result<DataSetReader, String> {
        Ok(DataSetReader {
            grib_readers: {

                let folders = result_or_return_why!(fs::read_dir(self.dataset_directory.as_str()), "Could not read dir");

                // figure out which data directory to read from
                let mut best_date = 0;
                let mut best_dir : Option<DirEntry> = None;

                for path in folders {
                    let dir = result_or_return_why!(path, "Could not read path");

                    let file_name = dir.file_name();
                    let name = some_or_return_why!(file_name.to_str(), "Could not read filename");

                    let date_num = name.parse::<i32>();

                    match date_num {
                        Ok(val) => {
                            if val > best_date {
                                best_date = val;
                                best_dir = Some(dir);
                            }
                        }
                        Err(_) => {
                            // println!("Warning: junk file/folder in lib/data: {}", name)
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

                let files = result_or_return_why!(fs::read_dir(
                    some_or_return_why!(best_dir, "Could not read dir").path()
                ), "Could not read dir");

                for path in files {
                    let file = result_or_return_why!(path, "Could not get path");
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
                    let name = some_or_return_why!(file_name.to_str(), "Could not read filename");

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

                    let extension = some_or_return_why!(
                        some_or_return_why!(path.extension(), "Could not read extenstion").to_str(),
                        "Could not read extension"
                    );

                    if extension == "grb2" {
                        let reader = GribReader::new(
                            some_or_return_why!(path.to_str(), "Could not read path").to_string()
                        );

                        readers.push(Box::new(result_or_return!(reader)));
                    }
                }

                // TODO: enforce reader sort order

                // println!("Readers initialized: {}ms", Utc::now().signed_duration_since(start_time).num_milliseconds());

                readers
            }
        })
    }
}

impl DataSetReader {

    pub fn velocity_at(&mut self, point: &Point) -> Result<Velocity, String> {
        match self.get_reader(point) {
            Ok(reader) => {
                reader.velocity_at(point)
            },
            Err(why) => Err(why)
        }
    }

    pub fn get_datasets(&self) -> Result<Vec<DateTime<Utc>>, String> {
        let mut result = vec![];

        let readers = &self.grib_readers;

        for i in 0..readers.len() {
            let reader = &readers[i];

            result.push(reader.time.clone());
        }

        Ok(result)
    }

    fn get_reader(&mut self, point: &Point) -> Result<&mut Box<GribReader>, String> {
        // TODO: implement a binary search tree or alternative fast lookup

        let readers = &mut self.grib_readers;

        if readers.is_empty() {
            return Err(String::from("No grib readers"));
        }

        let mut best_index = 0;
//        let mut best_seconds = {
//            let best_reader = &readers[0];
//            best_reader.time.signed_duration_since(point.time).num_seconds().abs()
//        };
//
//        for i in 1..readers.len() {
//            let reader = &mut readers[i];
//
//            let abs_seconds = reader.time.signed_duration_since(point.time).num_seconds().abs();
//
//            if abs_seconds < best_seconds {
//                best_index = i;
//                best_seconds = abs_seconds;
//            }
//        }

        Ok(&mut readers[best_index])
    }
}

struct WrappedDataSetReader {
    dataset_directory: String,
    reader: Option<DataSetReader>
}

macro_rules! get_reader_then {
    ($sel:ident.$F:ident $( $arg:ident ),* ) => {

        #[allow(unused_mut)]
        match $sel.initialize_reader() {
            Ok(_) => {
                // take the reader temporarily
                let reader = $sel.reader.take();

                // unwrap will not panic, because we already know it has initialized properly
                let mut unwrapped = some_or_return_why!(reader, "No reader");

                let result = unwrapped.$F($($arg)*);

                // replace it
                $sel.reader = Some(unwrapped);

                result
            },
            Err(why) => Err(why)
        }
    };
}

impl WrappedDataSetReader {

    pub fn velocity_at(&mut self, point: &Point) -> Result<Velocity, String> {
        get_reader_then!(self.velocity_at point)
    }


    pub fn get_datasets(&mut self) -> Result<Vec<DateTime<Utc>>, String> {
        get_reader_then!(self.get_datasets)
    }

    /*
     * Function to create a new dataset reader if needed
     */
    fn initialize_reader(&mut self) -> Result<bool, String> {

        if !self.reader.is_none() {
            return Ok(true);
        }

        match self.create() {
            Ok(reader) => {
                self.reader = Some(reader);
                Ok(true)
            },
            Err(why) => {
                Err(why)
            }
        }
    }

    pub fn new(dataset_directory : String) -> Self {
        WrappedDataSetReader {
            dataset_directory: dataset_directory,
            reader: None
        }
    }

    fn create(&self) -> Result<DataSetReader, String> {
        let mut uninitialized = UninitializedDataSetReader {
            dataset_directory: self.dataset_directory.clone()
        };

        uninitialized.initialize()
    }
}

lazy_static! {
    static ref READER : Mutex<WrappedDataSetReader> = Mutex::new(WrappedDataSetReader::new(
        [env::var("RAILS_ROOT").expect("RAILS_ROOT environment variable not found"), "/lib/data".to_string()].concat()
    ));
}

pub fn velocity_at(point: &Point) -> Result<Velocity, String> {
    let result = result_or_return_why!(READER.lock(), "Could not establish lock on reader").velocity_at(&point);

    result
}

pub fn get_datasets() -> Result<Vec<DateTime<Utc>>, String> {
    let result = result_or_return_why!(READER.lock(), "Could not establish lock on reader").get_datasets();

    result
}
