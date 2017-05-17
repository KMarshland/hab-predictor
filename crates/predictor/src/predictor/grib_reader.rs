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
    grid_definition: LatLonGridDefinition
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
    earth_model: u64, // see http://www.nco.ncep.noaa.gov/pmb/docs/grib2/grib2_table3-2.shtml

    radius_scale_factor: u64,
    radius_scale_value: u64,

    major_axis_scale_factor: u64,
    major_axis_scale_value: u64,

    minor_axis_scale_factor: u64,
    minor_axis_scale_value: u64,

    points_along_parallel: u64,
    points_along_meridian: u64,

    basic_angle: u64,
    basic_angle_subdivision: u64,

    first_lat: u64,
    first_lon: u64,

    resolution_flags: u64,

    last_lat: u64,
    last_lon: u64,

    delta_i: u64,
    delta_j: u64,

    scanning_mode: u64
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

        let file = File::open(self.get_path()).unwrap();

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

        let section_3_head_length = self.bytes_read;

        // 15-xx. Grid definition template (See Template 3.N, where N is the grid definition template number given in octets 13-14)
        let grid_definition : LatLonGridDefinition=  match grid_definition_number{
            0 => {
                // 15. Shape of the Earth (See Code Table 3.2)
                let earth_shape = self.read_as_number(1);

                // 16. Scale Factor of radius of spherical Earth
                let radius_scale_factor = self.read_as_number(1);

                // 17-20. Scale value of radius of spherical Earth
                let radius_scale_value = self.read_as_number(4);

                // 21. Scale factor of major axis of oblate spheroid Earth
                let major_axis_scale_factor = self.read_as_number(1);

                // 22-25. Scaled value of major axis of oblate spheroid Earth
                let major_axis_scale_value = self.read_as_number(4);

                // 26. Scale factor of minor axis of oblate spheroid Earth
                let minor_axis_scale_factor = self.read_as_number(1);

                // 27-30. Scaled value of minor axis of oblate spheroid Earth
                let minor_axis_scale_value = self.read_as_number(4);

                // 31-34. Ni—number of points along a parallel
                let points_along_parallel = self.read_as_number(4);

                // 35-38. Nj—number of points along a meridian
                let points_along_meridian = self.read_as_number(4);

                // 39-42. Basic angle of the initial production domain (see Note 1)
                let basic_angle = self.read_as_number(4);

                // 43-46. Subdivisions of basic angle used to define extreme longitudes and latitudes, and direction increments (see Note 1)
                let basic_angle_subdivision = self.read_as_number(4);

                // 47-50. La1—latitude of first grid point (see Note 1)
                let first_lat = self.read_as_number(4);

                // 51-54. Lo1—longitude of first grid point (see Note 1)
                let first_lon = self.read_as_number(4);

                // 55. Resolution and component flags (see Flag Table 3.3)
                let resolution_flags = self.read_as_number(1);

                // 56-59. La2—latitude of last grid point (see Note 1)
                let last_lat = self.read_as_number(4);

                // 60-63. Lo2—longitude of last grid point (see Note 1)
                let last_lon = self.read_as_number(4);

                // 64-67. Di—i direction increment (see Notes 1 and 5)
                let delta_i = self.read_as_number(4);

                // 68-71. Dj—j direction increment (see Note 1 and 5)
                let delta_j = self.read_as_number(4);

                // 72. Scanning mode (flags — see Flag Table 3.4 and Note 6)
                let scanning_mode = self.read_as_number(1);

                LatLonGridDefinition {
                    earth_model: earth_shape,

                    radius_scale_factor: radius_scale_factor,
                    radius_scale_value: radius_scale_value,

                    major_axis_scale_factor: major_axis_scale_factor,
                    major_axis_scale_value: major_axis_scale_value,

                    minor_axis_scale_factor: minor_axis_scale_factor,
                    minor_axis_scale_value: minor_axis_scale_value,

                    points_along_parallel: points_along_parallel,
                    points_along_meridian: points_along_meridian,

                    basic_angle: basic_angle,
                    basic_angle_subdivision: basic_angle_subdivision,

                    first_lat: first_lat,
                    first_lon: first_lon,

                    resolution_flags: resolution_flags,

                    last_lat: last_lat,
                    last_lon: last_lon,

                    delta_i: delta_i,
                    delta_j: delta_j,

                    scanning_mode: scanning_mode
                }
            },
            _ => {
                return Result::Err(String::from("Can only parse standard grid definition (grid definition error)"));
            }
        };

        // [xx+1]-nn. Optional list of numbers defining number of points (See notes 2, 3, and 4 below)
        let head_length = section_3_head_length - self.bytes_read;
        self.read_n(section_3_length - 14 - head_length);

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

            list_interpretation: list_interpretation,
            grid_definition: grid_definition
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
