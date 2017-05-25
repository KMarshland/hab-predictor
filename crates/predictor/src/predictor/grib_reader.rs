use std::io::prelude::*;
use std::fs::File;
use std::mem;
use chrono::prelude::*;
use predictor::point::*;

struct UnprocessedGribReader {
    path: String
}

struct ProcessingGribReader {
    file: File,
    bytes_read: u64
}

#[allow(dead_code)]
pub struct GribReader {
    reference_time: ReferenceTime,
    pub time: DateTime<UTC>,
}

enum ReferenceTime {
    Analysis,
    StartOfForecast,
    VerifyingTimeOfForecast,
    ObservationTime,
    Invalid
}

impl UnprocessedGribReader {

    fn read(&mut self) -> Result<GribReader, String> {
        println!("Trying to read from {}", self.get_path());

        let file = File::open(self.get_path()).unwrap();

        let mut reader = ProcessingGribReader {
            bytes_read: 0,
            file: file
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

        println!("File is Edition {}", edition);

        // 9-16. Total length of GRIB message in octets (All sections)
        let total_length = self.read_as_u64(8);


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

        // 20. Production Status of Processed data in the GRIB message (See Table 1.3)
        self.read_n(1);

        // 21. Type of processed data in this GRIB message (See Table 1.4)
        self.read_n(1);

        // 22-N. Reserved
        self.read_n(section_1_length - 21);

        Ok(GribReader {
            reference_time: reference_time,
            time: UTC.ymd(year as i32, month as u32, day as u32).and_hms(hour as u32, minute as u32, second as u32),
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

        let mut number : u64 = 0;
        for i in 0..number_of_bytes {
            // TODO: these casts spook me
            number += (buf[i as usize] as u64) << (8 * (number_of_bytes - i - 1));
        }

        number
    }

    fn get_file(&mut self) -> &mut File {
        &mut self.file
    }
}
