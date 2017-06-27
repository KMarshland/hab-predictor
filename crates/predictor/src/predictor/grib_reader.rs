use std::io::prelude::*;
use std::fs::File;
use std::mem;
use chrono::prelude::*;
use predictor::point::*;
use lru_cache::LruCache;

const CELL_SIZE : f32 = 25.0; // Make sure this matches the grid size in grib_convert.rb
const CACHE_SIZE : usize = 50_000; // in velocity tuples

struct UnprocessedGribReader {
    path: String
}

struct ProcessingGribReader {
    path: String,
    file: File,
    bytes_read: u64
}

#[allow(dead_code)]
pub struct GribReader {
    reference_time: ReferenceTime,
    pub time: DateTime<UTC>,

    path: String,

    cache: LruCache<u32, Velocity>
}

enum ReferenceTime {
    Analysis,
    StartOfForecast,
    VerifyingTimeOfForecast,
    ObservationTime,
    Invalid
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

impl GribReader {

    pub fn new(path: String) -> GribReader {
        let mut reader = UnprocessedGribReader {
            path: path
        };

        reader.read().unwrap()
    }

    /*
     * Returns the interpolated velocity at a given point
     */
    pub fn velocity_at(&mut self, point: &Point) -> Result<Velocity, String> {

        // get the four points to interpolate between
        let aligned = point.align();
        let ne = result_or_return!(self.velocity_at_aligned(&aligned.ne));
        let nw = result_or_return!(self.velocity_at_aligned(&aligned.nw));
        let se = result_or_return!(self.velocity_at_aligned(&aligned.se));
        let sw = result_or_return!(self.velocity_at_aligned(&aligned.sw));

        // lerp lerp lerp
        Ok(
            (ne * aligned.percent_east + &(nw * aligned.percent_west)) * aligned.percent_north +
                &((se * aligned.percent_east + &(sw * aligned.percent_west)) * aligned.percent_south)
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
            parts.next().unwrap().to_string() +
                "/L" + aligned.level.to_string().as_str() +
                "/C" + grid_lat.to_string().as_str() + "_" + grid_lon.to_string().as_str() +
                ".gribp"
        };

        self.scan_file(proper_filename, aligned)
    }

    /*
     *
     */
    fn scan_file(&mut self, filename : String, aligned : &AlignedPoint) -> Result<Velocity, String> {
        let name = &filename;
        let mut file = &mut File::open(name).unwrap();

        let mut u : f32 = 0.0;
        let mut v : f32 = 0.0;

        let mut data_found = false;

        loop {
            match GribReader::read_line(&mut file) {
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
                            north: line.v,
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
            east: u,
            north: v,
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


impl UnprocessedGribReader {

    fn read(&mut self) -> Result<GribReader, String> {
        let file = File::open(self.get_path()).unwrap();

        let mut reader = ProcessingGribReader {
            bytes_read: 0,
            file: file,
            path: self.get_path().to_string()
        };

        let result = reader.read();

        result
    }

    pub fn get_path(&self) -> &str {
        &self.path
    }
}

impl ProcessingGribReader {

    /*
     * Parses the GRIB file so that it can read actual data
     * See http://www.nco.ncep.noaa.gov/pmb/docs/grib2/grib2_doc.shtml for format documentation
     */
    fn read(&mut self) -> Result<GribReader, String> {
        let mut buf : Vec<u8>;


        /*
         * SECTION 0: INDICATOR SECTION
         */

        // 1-4. GRIB
        buf = result_or_return!(self.read_n(4));

        if buf[0] != 'G' as u8 || buf[1] != 'R' as u8 || buf[2] != 'I' as u8 || buf[3] != 'B' as u8 {
            return Result::Err(String::from("Incorrect header (expected GRIB)"))
        }

        // 5-7. Total length
        result_or_return!(self.read_n(3));

        // 8. Edition number
        buf = result_or_return!(self.read_n(1));
        let edition = buf[0];

        if edition != 2 {
            return Result::Err(String::from("Incorrect edition (expected version 2)"))
        }

        // 9-16. Total length of GRIB message in octets (All sections)
        result_or_return!(self.read_n(8));


        /*
         * SECTION 1: IDENTIFICATION SECTION
         */

        // 1-4. Length of the section in octets (21 or N)
        let section_1_length = result_or_return!(self.read_as_u64(4));
        if section_1_length < 21 {
            return Result::Err(String::from("Section 1 too short (length: ".to_string() +
                section_1_length.to_string().as_str() + ")"))
        }

        // 5. Number of the section (1)
        let section_1_number = result_or_return!(self.read_as_u64(1));
        if section_1_number != 1 {
            return Result::Err(String::from("Incorrect section number (expected 1)"))
        }

        // 6-7. Identification of originating/generating center
        result_or_return!(self.read_n(2));

        // 8-9. Identification of originating/generating subcenter
        result_or_return!(self.read_n(2));

        // 10. GRIB master tables version number (currently 2)
        let table_version_number = result_or_return!(self.read_as_u64(1));
        if table_version_number != 2 {
            return Result::Err(String::from("Incorrect GRIB master tables version (expected 2, got ".to_string() +
                table_version_number.to_string().as_str() + ")"));
        }

        // 11. Version number of GRIB local tables used to augment Master Tables
        result_or_return!(self.read_n(1));

        // 12. Significance of reference time
        let reference_time = match result_or_return!(self.read_as_u64(1)) {
            0 => ReferenceTime::Analysis,
            1 => ReferenceTime::StartOfForecast,
            2 => ReferenceTime::VerifyingTimeOfForecast,
            3 => ReferenceTime::ObservationTime,
            _ => ReferenceTime::Invalid
        };

        // 13-14. Year (4 digits)
        let year = result_or_return!(self.read_as_u64(2));

        // 15. Month
        let month = result_or_return!(self.read_as_u64(1));

        // 16. Day
        let day = result_or_return!(self.read_as_u64(1));

        // 17. Hour
        let hour = result_or_return!(self.read_as_u64(1));

        // 18. Minute
        let minute = result_or_return!(self.read_as_u64(1));

        // 19. Second
        let second = result_or_return!(self.read_as_u64(1));

        let time = UTC.ymd(year as i32, month as u32, day as u32).and_hms(hour as u32, minute as u32, second as u32);

        Ok(GribReader {
            reference_time: reference_time,
            time: time,
            path: self.path.clone(),
            cache: LruCache::new(CACHE_SIZE)
        })
    }

    fn read_n(&mut self, number_of_bytes : u64) -> Result<Vec<u8>, String> {
        let mut buf = vec![];
        if number_of_bytes <= 0 {
            return Ok(buf);
        }

        {
            let mut handle = self.get_file().take(number_of_bytes);
            handle.read_to_end(&mut buf).unwrap();
        }

        if buf.len() as u64 != number_of_bytes {
            return Err("Only read ".to_string() + buf.len().to_string().as_str() +
                " bytes, expected to read " + number_of_bytes.to_string().as_str())
        }

        self.bytes_read += number_of_bytes;

        Ok(buf)
    }

    fn read_as_u64(&mut self, number_of_bytes : u64) -> Result<u64, String> {
        if number_of_bytes <= 0 {
            return Ok(0);
        }

        let buf = result_or_return!(self.read_n(number_of_bytes));

        Ok(bytes_to_u64(buf, number_of_bytes))
    }

    fn get_file(&mut self) -> &mut File {
        &mut self.file
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

