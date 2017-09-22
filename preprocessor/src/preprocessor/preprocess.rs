use std::env;
use std::path;
use std::fs;
use std::ffi::OsStr;
use std::process::{Command, Stdio};
use std::io::{BufReader, BufRead};

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

    let processed_dir = env::current_dir().unwrap().join("data").join("processed").join(at.format("%Y%m%d").to_string());
    fs::create_dir_all(&processed_dir)?;

    for entry in fs::read_dir(raw_data_directory)? {
        let path = entry?.path();
        let extension = path.extension();

        if extension != Some(OsStr::new("grb2")) {
            continue;
        }

        process_file(&path, &processed_dir)?;
        break
    }

    Ok(())
}

/*
 * Processes the provided grib file
 */
fn process_file(file : &path::PathBuf, processed_dir : &path::PathBuf) -> Result<(), PreprocessorError> {
    println!("Processing {}", file.to_string_lossy());

    let output_dir = processed_dir.join(file.file_stem().unwrap());
    let partial_dir = (&output_dir).with_extension("partial");

    // remove any partial data in order to reprocess
    if (&partial_dir).exists() {
        fs::remove_dir_all(&output_dir.with_extension("partial"))?;
    }

    // recreate the partial directory
    fs::create_dir_all(&partial_dir)?;

    let mut process = Command::new("grib_get_data")
        .arg("-p").arg("shortName,level")
        .arg("-w").arg("shortName=u/v/t,level!=0")
        .arg(file)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn().unwrap();

    let reader = BufReader::new(process.stdout.unwrap());

    let mut lines_iter = reader.lines();
    loop {
        match lines_iter.next() {
            Some(line) => {

            },
            None => {
                println!("None!");
                break;
            }
        }
    }

    Ok(())
}
