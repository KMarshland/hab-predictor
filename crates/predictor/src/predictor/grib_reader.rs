use std::io::prelude::*;
use std::fs::File;
use std::mem;
use chrono::prelude::*;
use predictor::point::*;

const CELL_SIZE : f32 = 25.0; // Make sure this matches the grid size in grib_convert.rb
const DATA_RESOLUTION : f32 = 0.5;

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

    path: String
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
    value : f32,
    key : char
}

impl GribReader {

    pub fn new(path: String) -> GribReader {
        let mut reader = UnprocessedGribReader {
            path: path
        };

        reader.read().unwrap()
    }

    pub fn velocity_at(&self, point: &Point) -> Velocity {
        let isobaric_hpa = 1013.25*(1.0 - point.altitude/44330.0).powf(5.255);

        //TODO: make a fast lookup structure for this
        let levels = [2, 3, 5, 7, 10, 20, 30, 50, 70, 100, 150, 200, 250, 300, 350, 400, 450, 500, 550, 600, 650, 700, 750, 800, 850, 900, 925, 950, 975, 1000];
        let mut best_level : i32 = 1;
        let mut best_level_diff : f32 = (isobaric_hpa - (best_level as f32)).abs();

        for level_ref in levels.iter() {
            let level = *level_ref as i32;
            let diff = (isobaric_hpa - (level as f32)).abs();

            if diff < best_level_diff {
                best_level = level;
                best_level_diff = diff;
            }
        }

        // Round to nearest DATA_RESOLUTION
        let lat = (point.latitude / DATA_RESOLUTION).round() * DATA_RESOLUTION;
        let lon = (point.longitude / DATA_RESOLUTION).round() * DATA_RESOLUTION + 180.0;

        let grid_lat = (point.latitude / CELL_SIZE).floor() * CELL_SIZE;
        let grid_lon = ((point.longitude + 180.0) / CELL_SIZE).floor() * CELL_SIZE;

        let proper_filename = {
            let mut parts = self.path.split('.');
            parts.next().unwrap().to_string() +
                "/L" + best_level.to_string().as_str() +
                "/C" + grid_lat.to_string().as_str() + "_" + grid_lon.to_string().as_str() +
                ".gribp"
        };

        GribReader::scan_file(proper_filename, lat, lon)
    }

    fn scan_file(filename : String, lat : f32, lon : f32) -> Velocity {
        let name = &filename;
        let mut file = &mut File::open(name).unwrap();

        let mut u : f32 = 0.0;
        let mut v : f32 = 0.0;

        let mut has_u = false;
        let mut has_v = false;


        loop {
            match GribReader::read_line(&mut file) {
                Some(line) => {

                    if lat == line.lat && lon == line.lon {
                        match line.key {
                            'u' => {
                                u = line.value;
                                has_u = true;
                            },
                            'v' => {
                                v = line.value;
                                has_v = true;
                            },
                            _ => {
                                println!("Unknown key: {}", line.key)
                            }
                        };

                        if has_u && has_v {
                            break;
                        }
                    }
                }
                None => {
                    println!("Trying to find: {},{}", lat, lon);
                    println!("Searching in {}", name);

                    panic!("Reached end without finding data");
                }
            }
        }

        Velocity {
            north: u,
            east: v,
            vertical: 0.0
        }
    }

    /*
     * Reads a line into a struct
     * All values except for the key are IEEE754 formatted floats
     * The key is just a byte
     */
    fn read_line(file: &mut File) -> Option<GribLine> {

        let mut buffer = [0; 13];

        match file.read(&mut buffer) {
            Ok(bytes) => {
                if bytes == 0 { //EOF
                    return None;
                }

                if bytes != 13 {
                    panic!("Invalid number of bytes: ".to_string() + bytes.to_string().as_str());
                }
            },
            Err(why) => {
                println!("{:?}", why);
                return None;
            }
        }

        let lat = bytes_to_f32(buffer[0..4].to_vec());
        let lon = bytes_to_f32(buffer[4..8].to_vec());
        let val = bytes_to_f32(buffer[8..12].to_vec());
        let key = buffer[12] as char;

        Some(GribLine {
            lat: lat,
            lon: lon,
            value: val,
            key: key
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
        buf = self.read_n(4);

        if buf[0] != 'G' as u8 || buf[1] != 'R' as u8 || buf[2] != 'I' as u8 || buf[3] != 'B' as u8 {
            return Result::Err(String::from("Incorrect header (expected GRIB)"))
        }

        // 5-7. Total length
        self.read_n(3);

        // 8. Edition number
        buf = self.read_n(1);
        let edition = buf[0];

        if edition != 2 {
            return Result::Err(String::from("Incorrect edition (expected version 2)"))
        }

        // 9-16. Total length of GRIB message in octets (All sections)
        self.read_as_u64(8);


        /*
         * SECTION 1: IDENTIFICATION SECTION
         */

        // 1-4. Length of the section in octets (21 or N)
        let section_1_length = self.read_as_u64(4);
        if section_1_length < 21 {
            return Result::Err(String::from("Section 1 too short (length: ".to_string() +
                section_1_length.to_string().as_str() + ")"))
        }

        // 5. Number of the section (1)
        let section_1_number = self.read_as_u64(1);
        if section_1_number != 1 {
            return Result::Err(String::from("Incorrect section number (expected 1)"))
        }

        // 6-7. Identification of originating/generating center
        self.read_n(2);

        // 8-9. Identification of originating/generating subcenter
        self.read_n(2);

        // 10. GRIB master tables version number (currently 2)
        let table_version_number = self.read_as_u64(1);
        if table_version_number != 2 {
            return Result::Err(String::from("Incorrect GRIB master tables version (expected 2, got ".to_string() +
                table_version_number.to_string().as_str() + ")"));
        }

        // 11. Version number of GRIB local tables used to augment Master Tables
        self.read_n(1);

        // 12. Significance of reference time
        let reference_time = match self.read_as_u64(1) {
            0 => ReferenceTime::Analysis,
            1 => ReferenceTime::StartOfForecast,
            2 => ReferenceTime::VerifyingTimeOfForecast,
            3 => ReferenceTime::ObservationTime,
            _ => ReferenceTime::Invalid
        };

        // 13-14. Year (4 digits)
        let year = self.read_as_u64(2);

        // 15. Month
        let month = self.read_as_u64(1);

        // 16. Day
        let day = self.read_as_u64(1);

        // 17. Hour
        let hour = self.read_as_u64(1);

        // 18. Minute
        let minute = self.read_as_u64(1);

        // 19. Second
        let second = self.read_as_u64(1);

        let time = UTC.ymd(year as i32, month as u32, day as u32).and_hms(hour as u32, minute as u32, second as u32);

        Ok(GribReader {
            reference_time: reference_time,
            time: time,
            path: self.path.clone()
        })
    }

    fn read_n(&mut self, number_of_bytes : u64) -> Vec<u8> {
        let mut buf = vec![];
        if number_of_bytes <= 0 {
            return buf;
        }

        {
            let mut handle = self.get_file().take(number_of_bytes);
            handle.read_to_end(&mut buf).unwrap();
        }

        if buf.len() as u64 != number_of_bytes {
            panic!("Only read ".to_string() + buf.len().to_string().as_str() +
                " bytes, expected to read " + number_of_bytes.to_string().as_str())
        }

        self.bytes_read += number_of_bytes;

        buf
    }

    fn read_as_u64(&mut self, number_of_bytes : u64) -> u64 {
        if number_of_bytes <= 0 {
            return 0;
        }

        let buf = self.read_n(number_of_bytes);

        bytes_to_u64(buf, number_of_bytes)
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

fn bytes_to_f32(bytes : Vec<u8>) -> f32 {
    if bytes.len() != 4 {
        panic!("Invalid byte length ".to_string() + bytes.len().to_string().as_str())
    }

    let num = bytes_to_u64(bytes, 4) as u32;

    unsafe {mem::transmute(num)}
}

