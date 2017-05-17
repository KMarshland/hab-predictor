use std::io::prelude::*;
use std::fs::File;

struct UnprocessedGribReader {
    path: String,

    section_2_present: bool
}

struct ProcessingGribReader {
    file: File,
    bytes_read: u64,

    section_2_present: bool
}

#[allow(dead_code)]
pub struct GribReader {
    edition: u8,
    total_length: u64,

    reference_time: ReferenceTime,
    year: u64,
    month: u64,
    day: u64,
    hour: u64,
    minute: u64,
    second: u64,

    list_interpretation: Section3Interpretation,
}

enum ReferenceTime {
    Analysis,
    StartOfForecast,
    VerifyingTimeOfForecast,
    ObservationTime,
    Invalid
}

#[derive(PartialEq)]
enum Section3Interpretation {
    None,
    ParallelCount,
    ExtremaCount,
    Latitudes,
    Invalid
}

// http://www.nco.ncep.noaa.gov/pmb/docs/grib2/grib2_temp3-0.shtml
#[allow(dead_code)]
struct LatLonGridDefinition {

}


impl GribReader {

    pub fn new(path: String) -> GribReader {
        let mut reader = UnprocessedGribReader {
            path: path,
            section_2_present: false
        };

        reader.read().unwrap()
    }

}

impl UnprocessedGribReader {

    fn read(&mut self) -> Result<GribReader, String> {
        println!("Trying to read from {}", self.get_path());

        let mut file = File::open(self.get_path()).unwrap();

        let mut reader = ProcessingGribReader {
            bytes_read: 0,
            file: file,

            section_2_present: self.section_2_present
        };

        let result = reader.read();

        println!("Successfully read {}", self.get_path());

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
        let total_length = self.read_as_number(8);


        /*
         * SECTION 1: IDENTIFICATION SECTION
         */

        // 1-4. Length of the section in octets (21 or N)
        let section_1_length = self.read_as_number(4);
        if section_1_length < 21 {
            return Result::Err(String::from("Section 1 too short (length: ".to_string() +
                section_1_length.to_string().as_str() + ")"))
        }

        // 5. Number of the section (1)
        let section_1_number = self.read_as_number(1);
        if section_1_number != 1 {
            return Result::Err(String::from("Incorrect section number (expected 1)"))
        }

        // 6-7. Identification of originating/generating center
        self.read_n(2);

        // 8-9. Identification of originating/generating subcenter
        self.read_n(2);

        // 10. GRIB master tables version number (currently 2)
        let table_version_number = self.read_as_number(1);
        if table_version_number != 2 {
            return Result::Err(String::from("Incorrect GRIB master tables version (expected 2, got ".to_string() +
                table_version_number.to_string().as_str() + ")"));
        }

        // 11. Version number of GRIB local tables used to augment Master Tables
        self.read_n(1);

        // 12. Significance of reference time
        let reference_time = match self.read_as_number(1) {
            0 => ReferenceTime::Analysis,
            1 => ReferenceTime::StartOfForecast,
            2 => ReferenceTime::VerifyingTimeOfForecast,
            3 => ReferenceTime::ObservationTime,
            _ => ReferenceTime::Invalid
        };

        // 13-14. Year (4 digits)
        let year = self.read_as_number(2);

        // 15. Month
        let month = self.read_as_number(1);

        // 16. Day
        let day = self.read_as_number(1);

        // 17. Hour
        let hour = self.read_as_number(1);

        // 18. Minute
        let minute = self.read_as_number(1);

        // 19. Second
        let second = self.read_as_number(1);

        // 20. Production Status of Processed data in the GRIB message (See Table 1.3)
        self.read_n(1);

        // 21. Type of processed data in this GRIB message (See Table 1.4)
        self.read_n(1);

        // 22-N. Reserved
        self.read_n(section_1_length - 21);


        /*
         * SECTION 2: LOCAL USE SECTION
         */

        if self.section_2_present {
            // 1-4. Length of the section in octets (N)
            let section_2_length = self.read_as_number(4);
            if section_2_length < 5 {
                return Result::Err(String::from("Section 2 too short (length: ".to_string() +
                    section_2_length.to_string().as_str() + ")"))
            }

            // 5. Number of the section (2)
            let section_2_number = self.read_as_number(1);
            if section_2_number != 2 {
                return Result::Err(String::from("Incorrect section number (expected 2, got ".to_string() +
                    section_2_number.to_string().as_str() + ")"))
            }

            // 6-N. Local Use
            self.read_n(section_2_length - 5);
        }


        /*
         * SECTION 3: GRID DEFINITION SECTION
         */

        // 1-4. Length of the section in octets (N)
        let section_3_length = self.read_as_number(4);
        if section_3_length < 15 {
            return Result::Err(String::from("Section 3 too short (length: ".to_string() +
                section_3_length.to_string().as_str() + ")"))
        }

        // 5. Number of the section (3)
        let section_3_number = self.read_as_number(1);
        if section_3_number != 3 {
            return Result::Err(String::from("Incorrect section number (expected 3, got ".to_string() +
                section_3_number.to_string().as_str() + ")"))
        }

        // 6. Source of grid definition (See Table 3.0) (See note 1 below)
        let grid_definition_source = self.read_as_number(1);
        if grid_definition_source != 0 {
            return Result::Err(String::from("Can only parse standard grid definition (source error)"));
        }

        // 7-10. Number of data points
        let datapoint_count = self.read_as_number(4);

        // 11. Number of octets for optional list of numbers defining number of points (See note 2 below)
        let optional_octet_count = self.read_as_number(4);
        if optional_octet_count != 0 {
            return Result::Err(String::from("Can only parse standard grid definition (octet count error)"));
        }

        // 12. Interpetation of list of numbers defining number of points (See Table 3.11)
        let list_interpretation = match self.read_as_number(1) {
            0 => Section3Interpretation::None,
            1 => Section3Interpretation::ParallelCount,
            2 => Section3Interpretation::ExtremaCount,
            3 => Section3Interpretation::Latitudes,
            _ => Section3Interpretation::Invalid,
        };

        // 13-14. Grid definition template number (= N) (See Table 3.1)
        let grid_definition_number = self.read_as_number(2);
        if grid_definition_number != 0 {
            return Result::Err(String::from("Can only parse standard grid definition (grid definition error)"));
        }

        // 15-xx. Grid definition template (See Template 3.N, where N is the grid definition template number given in octets 13-14)
        let grid_definition : LatLonGridDefinition;


        // [xx+1]-nn. Optional list of numbers defining number of points (See notes 2, 3, and 4 below)
        self.read_n(section_3_length - 14);

        Ok(GribReader {
            edition: edition,
            total_length: total_length,

            reference_time: reference_time,
            year: year,
            month: month,
            day: day,
            hour: hour,
            minute: minute,
            second: second,

            list_interpretation: list_interpretation
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

        self.bytes_read += number_of_bytes;

        buf
    }

    fn read_as_number(&mut self, number_of_bytes : u64) -> u64 {
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
