use std::env;
use std::path;
use std::fs;
use std::ffi::OsStr;

use chrono::prelude::*;

use preprocessor::preprocessor_error::*;

/*
 * Preprocesses the dataset for the given time
 */
pub fn preprocess(at : DateTime<Utc>) -> Result<(), PreprocessorError> {

    let raw_data_directory = env::current_dir().unwrap().join("data").join("raw").join(at.format("%Y%m%d").to_string());

    if !raw_data_directory.exists() {
        return Err(PreprocessorError::InvalidDatasetError);
    }

    for entry in fs::read_dir(raw_data_directory)? {
        let path = entry?.path();
        let extension = path.extension();

        if extension != Some(OsStr::new("grb2")) {
            continue;
        }

        process_file(&path)?;
    }

    Ok(())
}

/*
 * Processes the provided grib file
 */
fn process_file(file : &path::PathBuf) -> Result<(), PreprocessorError> {
    println!("Processing {}", file.to_string_lossy());

    Ok(())
}
