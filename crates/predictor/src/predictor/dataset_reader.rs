use std::sync::Mutex;
use std::fs;
use std::env;
use std::fs::DirEntry;
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
                        Err(why) => println!("Warning: junk file/folder in lib/data: {}", name),
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
                let mut last_hour = 0.0;
                let mut last_file = &bucket[0];

                for file in &bucket[1..] {
                    println!("{}", file.path().display());

                    let name = file.file_name().into_string().unwrap();
                    let hour = name.split("_").collect::<Vec<&str>>()[4][0..3].parse::<f32>().unwrap();

                    let divider = (hour - (((hour-last_hour)-1.0)/2.0)).round() as i32;

                    for hr in (last_hour as i32)..divider {
                        let reader = Box::new(GribReader::new(last_file.path().to_str().unwrap().to_string()));
                        readers[hr as usize] = reader;
                    }

                    for hr in divider..(hour as i32) {
                        let reader = Box::new(GribReader::new(file.path().to_str().unwrap().to_string()));
                        readers[hr as usize] = reader;
                    }

                    last_hour = hour;
                    last_file = file;
                }

                // TODO: enforce reader sort order

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

        let first_reader = &self.grib_readers[0];
        // Number of hours since the start of the data
        let num_hours = (((first_reader.time.signed_duration_since(point.time).num_minutes().abs()) as f64)/60.0).round() as usize;

        let best_reader = &self.grib_readers.get(num_hours);

        match best_reader {
            &Some(reader) => return reader,
            _ => panic!("Error: Inputted time outside available time range for data"),
        }
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
