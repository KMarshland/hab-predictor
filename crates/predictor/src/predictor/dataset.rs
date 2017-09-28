use std::io::prelude::*;
use std::fs::File;
use std::mem;
use chrono::prelude::*;
use predictor::point::*;
use lru_cache::LruCache;

const CELL_SIZE : f32 = 25.0; // Make sure this matches the grid size in grib_convert.rb
const CACHE_SIZE : usize = 50_000; // in velocity tuples


#[allow(dead_code)]
pub struct Dataset {
    pub created_at: DateTime<Utc>,
    pub time: DateTime<Utc>,

    path: String,

    cache: LruCache<u32, Velocity>
}

struct GribLine {
    lat : f32,
    lon : f32,
    u : f32,
    v : f32
}

enum GribReadError {
    EOF,
    Corrupted(usize),
    IO(String)
}

impl Dataset {

    pub fn new(path: String) -> Result<Dataset, String> {
        Ok(Dataset {
            path,
            created_at: Utc::now(),
            time: Utc::now(),
            cache: LruCache::new(CACHE_SIZE)
        })
    }

    /*
     * Returns the interpolated velocity at a given point
     */
    pub fn velocity_at(&mut self, point: &Point) -> Result<Velocity, String> {

        // get the eight points to interpolate between
        let aligned = point.align();
        let ne_down = result_or_return!(self.velocity_at_aligned(&aligned.ne_down));
        let ne_up = result_or_return!(self.velocity_at_aligned(&aligned.ne_up));
        let nw_down = result_or_return!(self.velocity_at_aligned(&aligned.nw_down));
        let nw_up = result_or_return!(self.velocity_at_aligned(&aligned.nw_up));
        let se_down = result_or_return!(self.velocity_at_aligned(&aligned.se_down));
        let se_up = result_or_return!(self.velocity_at_aligned(&aligned.se_up));
        let sw_down = result_or_return!(self.velocity_at_aligned(&aligned.sw_down));
        let sw_up = result_or_return!(self.velocity_at_aligned(&aligned.sw_up));

        // lerp lerp lerp
        Ok(
            (
                (
                    ne_down * aligned.percent_east + &(nw_down * aligned.percent_west)
                ) * aligned.percent_north +

                    &(
                        (
                            se_down * aligned.percent_east + &(sw_down * aligned.percent_west)
                        ) * aligned.percent_south
                    )
            ) * aligned.percent_down +

                &(
                    (
                        (
                            ne_up * aligned.percent_east + &(nw_up * aligned.percent_west)
                        ) * aligned.percent_north +

                            &(
                                (
                                    se_up * aligned.percent_east + &(sw_up * aligned.percent_west)
                                ) * aligned.percent_south
                            )
                    ) * aligned.percent_up
                )
        )
    }

    /*
     * Returns the uninterpolated velocity at an aligned point
     */
    fn velocity_at_aligned(&mut self, aligned: &AlignedPoint) -> Result<Velocity, String> {
        // check cache
        {
            let ref mut cache = self.cache;

            match cache.get_mut(&aligned.key()) {
                Some(vel) => {
                    return Ok(vel.clone())
                },
                None => {}
            }
        }

        let grid_lat = (aligned.latitude / CELL_SIZE).floor() * CELL_SIZE;
        let grid_lon = (aligned.longitude / CELL_SIZE).floor() * CELL_SIZE;

        let proper_filename = {
            let mut parts = self.path.split('.');
            some_or_return_why!(parts.next(), "Could not get filename").to_string() +
                "/L" + aligned.level.to_string().as_str() +
                "/C" + grid_lat.to_string().as_str() + "_" + grid_lon.to_string().as_str() +
                ".gribp"
        };

        self.scan_file(proper_filename, aligned)
    }

    /*
     * Looks for a point in the file, adding everything in it to the cache
     * Note that u is east and v is south, as per https://en.wikipedia.org/wiki/Zonal_and_meridional
     */
    fn scan_file(&mut self, filename : String, aligned : &AlignedPoint) -> Result<Velocity, String> {
        let name = &filename;
        let mut file = &mut result_or_return_why!(File::open(name), "Could not open file");

        let mut u : f32 = 0.0;
        let mut v : f32 = 0.0;

        let mut data_found = false;

        loop {
            match Dataset::read_line(&mut file) {
                Ok(line) => {
                    if aligned.latitude == line.lat && aligned.longitude == line.lon {
                        u = line.u;
                        v = line.v;
                        data_found = true;
                    }

                    self.cache.insert(
                        AlignedPoint::cache_key(aligned.level, line.lat, line.lon),
                        Velocity {
                            east: line.u,
                            north: -line.v,
                            vertical: 0.0
                        }
                    );
                }
                Err(why) => {
                    match why {
                        GribReadError::EOF => {
                            break; // you're done!
                        },
                        GribReadError::Corrupted(_) => {
                            return Err(String::from("Invalid number of bytes in file"));
                        },
                        GribReadError::IO(why) => {
                            return Err(why);
                        }
                    }
                },
            }
        }

        if !data_found {
            println!("Looking for ({}, {}, {}) in {}",
                     aligned.latitude, aligned.longitude, aligned.level,
                     filename
            );
            return Err(String::from("Datapoint not found"));
        }

        Ok(Velocity {
            east: v,
            north: -u,
            vertical: 0.0
        })
    }

    /*
     * Reads a line into a struct
     * All values except for the key are IEEE754 formatted floats
     * The key is just a byte
     */
    fn read_line(file: &mut File) -> Result<GribLine, GribReadError> {

        // TODO: add a mem transmute thing here

        let mut buffer = [0; 16];

        match file.read(&mut buffer) {
            Ok(bytes) => {
                if bytes == 0 { //EOF
                    return Err(GribReadError::EOF);
                }

                if bytes != 16 {
                    return Err(GribReadError::Corrupted(bytes));
                }
            },
            Err(why) => {
                return Err(GribReadError::IO(why.to_string()));
            }
        }

        let lat = match bytes_to_f32(buffer[0..4].to_vec()) {
            Ok(val) => {
                val
            },
            Err(why) => {
                return Err(GribReadError::IO(why))
            }
        };

        let lon = match bytes_to_f32(buffer[4..8].to_vec()) {
            Ok(val) => {
                val
            },
            Err(why) => {
                return Err(GribReadError::IO(why))
            }
        };

        let u = match bytes_to_f32(buffer[8..12].to_vec()) {
            Ok(val) => {
                val
            },
            Err(why) => {
                return Err(GribReadError::IO(why))
            }
        };

        let v = match bytes_to_f32(buffer[12..16].to_vec()) {
            Ok(val) => {
                val
            },
            Err(why) => {
                return Err(GribReadError::IO(why))
            }
        };

        Ok(GribLine {
            lat: lat,
            lon: lon,
            u: u,
            v: v
        })
    }
}

fn bytes_to_u64(bytes : Vec<u8>, number_of_bytes : u64) -> u64 {
    let mut number : u64 = 0;
    for i in 0..number_of_bytes {
        // TODO: these casts spook me
        number += (bytes[i as usize] as u64) << (8 * (number_of_bytes - i - 1));
    }

    number
}

fn bytes_to_f32(bytes : Vec<u8>) -> Result<f32, String> {
    if bytes.len() != 4 {
        return Err("Invalid byte length ".to_string() + bytes.len().to_string().as_str())
    }

    let num = bytes_to_u64(bytes, 4) as u32;

    Ok(unsafe {mem::transmute(num)})
}
