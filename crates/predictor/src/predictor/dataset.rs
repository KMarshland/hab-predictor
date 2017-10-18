use std::io::prelude::*;
use std::fs::File;
use std::mem;

use chrono::prelude::*;
use chrono::Duration;

use predictor::point::*;
use predictor::dataset_reader::*;

const CELL_SIZE : f32 = 25.0; // Make sure this matches the grid size in grib_convert.rb


pub struct Dataset {
    pub created_at: DateTime<Utc>,
    pub time: DateTime<Utc>,

    id: u32,
    path: String,
    pub name: String
}

struct GribLine {
    latitude : f32,
    longitude : f32,
    u : f32,
    v : f32,
    temperature: f32
}

enum GribReadError {
    EOF,
    Corrupted(usize),
    IO(String)
}

impl Dataset {

    pub fn new(path: String, id: u32) -> Result<Dataset, String> {

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
            name, path, created_at, time, id
        })
    }

    /*
     * Returns the interpolated atmospheroid at a given point
     */
    pub fn atmospheroid_at(&self, point: &Point, cache: &mut Cache) -> Result<Atmospheroid, String> {

        // get the eight points to interpolate between
        let aligned = point.align();
        let ne_down = result_or_return!(self.atmospheroid_at_aligned(&aligned.ne_down, cache));
        let ne_up = result_or_return!(self.atmospheroid_at_aligned(&aligned.ne_up, cache));
        let nw_down = result_or_return!(self.atmospheroid_at_aligned(&aligned.nw_down, cache));
        let nw_up = result_or_return!(self.atmospheroid_at_aligned(&aligned.nw_up, cache));
        let se_down = result_or_return!(self.atmospheroid_at_aligned(&aligned.se_down, cache));
        let se_up = result_or_return!(self.atmospheroid_at_aligned(&aligned.se_up, cache));
        let sw_down = result_or_return!(self.atmospheroid_at_aligned(&aligned.sw_down, cache));
        let sw_up = result_or_return!(self.atmospheroid_at_aligned(&aligned.sw_up, cache));

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
     * Returns the uninterpolated atmospheroid at an aligned point
     */
    fn atmospheroid_at_aligned(&self, aligned: &AlignedPoint, cache: &mut Cache) -> Result<Atmospheroid, String> {
        // check cache
        {
            match cache.get_mut(&aligned.key(self.id)) {
                Some(atmo) => {
                    return Ok(atmo.clone())
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

        self.scan_file(proper_filename, aligned, cache)
    }

    /*
     * Looks for a point in the file, adding everything in it to the cache
     * Note that u is east and v is south, as per https://en.wikipedia.org/wiki/Zonal_and_meridional
     */
    fn scan_file(&self, filename : String, aligned : &AlignedPoint, cache: &mut Cache) -> Result<Atmospheroid, String> {
        let name = &filename;
        let mut file = &mut result_or_return_why!(File::open(name), "Could not open file");

        let mut u : f32 = 0.0;
        let mut v : f32 = 0.0;
        let mut temperature : f32 = 0.0;

        let mut data_found = false;

        loop {
            match Dataset::read_line(&mut file) {
                Ok(line) => {
                    if aligned.latitude == line.latitude && aligned.longitude == line.longitude {
                        u = line.u;
                        v = line.v;
                        temperature = line.temperature;
                        data_found = true;
                    }

                    cache.insert(
                        AlignedPoint::cache_key(aligned.level, line.latitude, line.longitude, self.id),
                        Atmospheroid {
                            velocity: Velocity {
                                east: line.u,
                                north: -line.v,
                                vertical: 0.0
                            },
                            temperature: line.temperature
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

        Ok(Atmospheroid {
            velocity: Velocity {
                east: v,
                north: -u,
                vertical: 0.0
            },
            temperature
        })
    }

    /*
     * Reads a line into a struct
     * All values except for the key are IEEE754 formatted floats
     * The key is just a byte
     */
    fn read_line(file: &mut File) -> Result<GribLine, GribReadError> {

        // TODO: add a mem transmute thing here

        let mut buffer = [0; 20];

        match file.read(&mut buffer) {
            Ok(bytes) => {
                if bytes == 0 { //EOF
                    return Err(GribReadError::EOF);
                }

                if bytes != 20 {
                    return Err(GribReadError::Corrupted(bytes));
                }
            },
            Err(why) => {
                return Err(GribReadError::IO(why.to_string()));
            }
        }

        let latitude = match bytes_to_f32(buffer[0..4].to_vec()) {
            Ok(val) => {
                val
            },
            Err(why) => {
                return Err(GribReadError::IO(why))
            }
        };

        let longitude = match bytes_to_f32(buffer[4..8].to_vec()) {
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

        let temperature = match bytes_to_f32(buffer[16..20].to_vec()) {
            Ok(val) => {
                val
            },
            Err(why) => {
                return Err(GribReadError::IO(why))
            }
        };

        Ok(GribLine {
            latitude,
            longitude,
            u,
            v,
            temperature
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
