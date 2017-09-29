use std::io::prelude::*;
use std::fs::File;
use std::mem;
use chrono::prelude::*;
use chrono::Duration;
use predictor::point::*;
use lru_cache::LruCache;

const CELL_SIZE : f32 = 25.0; // Make sure this matches the grid size in grib_convert.rb
const CACHE_SIZE : usize = 50_000; // in velocity tuples


#[allow(dead_code)]
pub struct Dataset {
    pub created_at: DateTime<Utc>,
    pub time: DateTime<Utc>,

    path: String,
    pub name: String,

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

        let (name, created_at, time) = {

            let name : &String = &some_or_return_why!(path.split("/").collect::<Vec<&str>>().last(), "Could not get name").to_string();

            let parts: Vec<&str> = name.split("_").collect();

            if parts.len() != 5 {
                return_error!(format!("Expected 5 parts in name, got {}", parts.len()));
            }

            if parts[4].contains(".") {
                return_error!("Is not a complete dataset");
            }

            if parts[0] != "gfs" {
                return_error!(format!("Expected first part to be gfs, got {}", parts[0]));
            }

            if parts[1] != "4" {
                return_error!(format!("Expected second part to be 4, got {}", parts[1]));
            }

            if parts[2].len() != 8 {
                return_error!(format!("Expected 8 characters in third part, got {}", parts[2].len()));
            }

            let year = match parts[2][0..4].parse::<i32>() {
                Ok(val) => val,
                Err(_) => {
                    return_error!("Invalid year");
                }
            };

            let month = match parts[2][4..6].parse::<u32>() {
                Ok(val) => val,
                Err(_) => {
                    return_error!("Invalid month");
                }
            };

            let day = match parts[2][6..8].parse::<u32>() {
                Ok(val) => val,
                Err(_) => {
                    return_error!("Invalid day");
                }
            };

            let hour = match parts[3] {
                "0000" => 0,
                "0600" => 6,
                "1200" => 12,
                "1800" => 18,
                _ => {
                    return_error!(format!("Invalid hour offset in fourth part: {}", parts[3]));
                }
            };

            let created_at = Utc.ymd(year, month, day).and_hms(hour, 0, 0);


            let hour_offset = match parts[4].parse::<u32>() {
                Ok(val) => val,
                Err(_) => {
                    return_error!("Invalid hour offset in fifth part");
                }
            };

            let time = created_at + Duration::hours(hour_offset as i64);

            (name.clone(), created_at, time)
        };

        Ok(Dataset {
            name, path, created_at, time,
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
