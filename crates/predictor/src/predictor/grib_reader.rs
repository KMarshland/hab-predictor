use std::io::prelude::*;
use std::fs::File;
use std::io::BufReader;
use std::process::Command;
use std::mem;
use chrono::prelude::*;
use predictor::point::*;

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
    key : String
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
            }
        }

        let lat = (point.latitude * 2.0).round() / 2.0;
        let lon = (point.longitude * 2.0).round() / 2.0 + 180.0; //TODO: make sure this is the right correction

        let proper_filename = {
            let mut parts = self.path.split('.');
            parts.next().unwrap().to_string() + "_l" + best_level.to_string().as_str() + ".gribp"
        };
        let proper_file = File::open(proper_filename).unwrap();

        GribReader::scan_file(proper_file, lat, lon)
    }

    fn scan_file(file : File, lat : f32, lon : f32) -> Velocity {
        let mut u : f32 = 0.0;
        let mut v : f32 = 0.0;

        let has_u = false;
        let has_v = false;

        let mut reader = BufReader::new(&file);

        loop {
            match GribReader::read_line(&mut reader) {
                _ => {
                    break;
                }
            }
        }

        Velocity {
            north: u,
            east: v,
            vertical: 0.0
        }
    }

    fn read_line(reader: &mut BufReader<&File>) -> Option<GribLine> {
        let mut buffer = vec![];

        let result = reader.read_until(b'\n', &mut buffer);
        match result {
            Ok(bytes) => {
                if
                    bytes < 12 {
                    return None;
                }
            },
            _ => {
                return None;
            }
        }

        let lat = bytes_to_f32(buffer[0..3].to_vec());
        let lon = bytes_to_f32(buffer[4..7].to_vec());
        let val = bytes_to_f32(buffer[7..10].to_vec());
        let key = String::from_utf8(buffer[11..].to_vec()).unwrap();

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
        println!("Trying to read from {}", self.get_path());

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

        self.convert();

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

    fn convert(&mut self){
        let command = "ruby lib/grib/convert.rb ".to_string() + self.path.as_str();
        println!("{}", command);

        let result = Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .expect("failed to execute process");

        //        println!("{}", String::from_utf8(result.stdout).unwrap());
        println!("{}", String::from_utf8(result.stderr).unwrap());
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
    let num = bytes_to_u64(bytes, 4) as u32;

    unsafe {mem::transmute(num)}
}

