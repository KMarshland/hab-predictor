use std::sync::Mutex;
use std::fs;
use std::env;

use lru_cache::LruCache;

use predictor::point::*;
use predictor::dataset::*;

const CACHE_SIZE : usize = 50_000_000; // in bytes
const BYTES_PER_CACHE_ELEMENT : usize = 16; // 4 floats

struct UninitializedDataSetReader {
    dataset_directory: String
}

pub type Cache = LruCache<u32, Atmospheroid>;

struct DataSetReader {
    cache: Cache,

    datasets: Vec<Box<Dataset>>
}

impl UninitializedDataSetReader {

    fn initialize(&mut self) -> Result<DataSetReader, String> {
        Ok(DataSetReader {
            cache: LruCache::new(CACHE_SIZE / BYTES_PER_CACHE_ELEMENT),

            datasets: {

                let mut readers : Vec<Box<Dataset>> = vec![];

                let folders = result_or_return_why!(fs::read_dir(self.dataset_directory.as_str()), "Could not read dir");

                for entry in folders {
                    let path = result_or_return_why!(entry, "Could not read entry").path();

                    let path_as_str = some_or_return_why!(path.to_str(), "Could not read path");

                    let reader = match Dataset::new(path_as_str.to_string()) {
                        Ok(reader) => reader,
                        Err(_) => {
                            continue;
                        }
                    };

                    readers.push(Box::new(reader));
                }

                readers
            }
        })
    }
}

impl DataSetReader {

    pub fn velocity_at(&mut self, point: &Point) -> Result<Velocity, String> {
        let atmospheroid = self.atmospheroid_at(point)?;

        Ok(atmospheroid.velocity)
    }

    pub fn atmospheroid_at(&mut self, point: &Point) -> Result<Atmospheroid, String> {
        let readers = &self.datasets;

        if readers.is_empty() {
            return Err(String::from("No grib readers"));
        }

        let mut best_index = 0;
        let mut best_seconds = {
            let best_reader = &readers[0];
            best_reader.time.signed_duration_since(point.time).num_seconds().abs()
        };

        for i in 1..readers.len() {
            let reader = &readers[i];

            let abs_seconds = reader.time.signed_duration_since(point.time).num_seconds().abs();

            if abs_seconds < best_seconds {
                best_index = i;
                best_seconds = abs_seconds;
            }
        }

        let reader = &readers[best_index];

        reader.atmospheroid_at(point, &mut self.cache)
    }

    pub fn get_datasets(&self) -> Result<Vec<String>, String> {
        let mut result = vec![];

        let readers = &self.datasets;

        for i in 0..readers.len() {
            let reader = &readers[i];

            result.push(reader.name.clone());
        }

        Ok(result)
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


    pub fn get_datasets(&mut self) -> Result<Vec<String>, String> {
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
            dataset_directory,
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
        [env::var("RAILS_ROOT").expect("RAILS_ROOT environment variable not found"), "/data".to_string()].concat()
    ));
}

pub fn velocity_at(point: &Point) -> Result<Velocity, String> {
    let result = result_or_return_why!(READER.lock(), "Could not establish lock on reader").velocity_at(&point);

    result
}

pub fn get_datasets() -> Result<Vec<String>, String> {
    let result = result_or_return_why!(READER.lock(), "Could not establish lock on reader").get_datasets();

    result
}
