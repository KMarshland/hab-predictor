use std::io;
use std::process::Command;

use chrono::prelude::*;
use chrono::Duration;
use reqwest;

use preprocessor::preprocessor_error::*;

/*
 * Finds out which dataset needs to be downloaded, then downloads it
 * Chooses the most recent dataset
 */
pub fn download() -> Result<(), PreprocessorError> {
    let mut at: DateTime<Utc> = Utc::now();
    let mut url : String;
    let client = reqwest::Client::new()?;

    loop {

        url = format!("https://nomads.ncdc.noaa.gov/data/gfs4/{}/{}/", at.format("%Y%m"), at.format("%Y%m%d"));

        println!("Checking dataset: {}", &url);

        if dataset_exists(&url)? {
            break;
        }

        at = at - Duration::days(1);
    }

    println!("Downloading dataset {}", url);

    Ok(())
}

/*
 * Checks to see if a dataset exists
 * Does so by making a HTTP request and checking that the status code is 200
 * Note: checks via curl, as reqwest hangs inexplicably
 */
fn dataset_exists(url : &String) -> Result<bool, io::Error> {
    let output = Command::new("curl")
        .arg("-I")
        .arg("-s")
        .arg("-o").arg("/dev/null")
        .arg("-w").arg("%{http_code}")
        .arg(url)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    Ok(stdout == "200")
}