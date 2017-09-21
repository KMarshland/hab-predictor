use std::io;
use std::process::Command;
use std::collections::VecDeque;
use std::cmp;
use std::sync::{Mutex, Arc};
use std::env;
use std::fs;

use chrono::prelude::*;
use chrono::Duration;
use reqwest;
use std::thread;

use preprocessor::preprocessor_error::*;

const PREDICTION_MAX_HOURS : u32 = 384;
const PREDICTION_PERIODS : &'static [&'static str] = &["0000", "0600", "1200", "1800"];
const HOUR_RESOLUTION : u32 = 3;
const DOWNLOAD_WORKERS : u32 = 25;

/*
 * Finds out which dataset needs to be downloaded, then downloads it
 * Chooses the most recent dataset
 */
pub fn download() -> Result<(), PreprocessorError> {
    let mut at: DateTime<Utc> = Utc::now();
    let mut url : String;

    loop {

        url = format!("https://nomads.ncdc.noaa.gov/data/gfs4/{}/{}/", at.format("%Y%m"), at.format("%Y%m%d"));

        println!("Checking dataset: {}", &url);

        if dataset_exists(&url)? {
            break;
        }

        at = at - Duration::days(1);
    }

    println!("Downloading dataset {}", url);
    download_dataset(at)?;

    Ok(())
}

/*
 * Checks to see if a dataset exists
 * Does so by making a HTTP request and checking that the status code is 200
 * Note: checks via curl, as reqwest hangs (seems like it strips the final /)
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

/*
 * Downloads the dataset for the given day
 */
fn download_dataset(at : DateTime<Utc>) -> Result<(), PreprocessorError> {

    // make the folder for the data to go in
    make_data_directory(at)?;

    // generate a list of all the urls you need to download
    let to_download = get_url_queue(at);

    // download them in a threadpool
    let worker_number = cmp::min(DOWNLOAD_WORKERS, to_download.len() as u32);
    let mut handles = vec![];

    let queue = Arc::new(Mutex::new(to_download));

    for i in 0..worker_number {
        let queue = queue.clone();
        let thread_number = i.clone();

        let handle = thread::spawn(move || {
            loop {
                let mut download_queue = queue.lock().unwrap();

                match download_queue.pop_front() {
                    Some(url) => {
                        println!("Thread {}: {}", thread_number, url);
                    },
                    None => {
                        break
                    }
                }
            }
        });
        handles.push(handle);
    }

    // wait for downloaders to finish
    for handle in handles {
        handle.join().unwrap();
    }

    Ok(())
}

/*
 * Sets up the directory for the data for the provided data
 */
fn make_data_directory(at : DateTime<Utc>) -> Result<(), io::Error> {
    let path = env::current_dir().unwrap().join("data").join("raw").join(at.format("%Y%m%d").to_string());
    fs::create_dir_all(path)?;

    Ok(())
}

fn get_url_queue(at : DateTime<Utc>) -> VecDeque<String> {
    let base_url = format!("https://nomads.ncdc.noaa.gov/data/gfs4/{}/{}", at.format("%Y%m"), at.format("%Y%m%d"));

    let mut to_download : VecDeque<String> = VecDeque::new();

    for period in PREDICTION_PERIODS {
        for hour_offset in 0..(PREDICTION_MAX_HOURS / HOUR_RESOLUTION) {
            let offset = format!("{:>03}", (hour_offset * HOUR_RESOLUTION));

            let url = format!("{}/gfs_4_{}_{}_{}.grb2", base_url, at.format("%Y%m%d"), period, offset);
            to_download.push_back(url);
        }
    }

    to_download
}

