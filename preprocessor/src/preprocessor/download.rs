use std::io;
use std::process::Command;
use std::collections::VecDeque;
use std::cmp;
use std::sync::{Mutex, Arc};
use std::env;
use std::fs;
use std::path;
use std::thread;

use chrono::prelude::*;
use chrono::Duration;

use preprocessor::preprocessor_error::*;

const PREDICTION_MAX_HOURS : u32 = 384;
const PREDICTION_PERIODS : &'static [&'static str] = &["0000", "0600", "1200", "1800"];
const HOUR_RESOLUTION : u32 = 3;
const DOWNLOAD_WORKERS : u32 = 10;

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

    println!("Downloading dataset {}\n  ", url);
    download_dataset(at)?;

    clean()?;

    Ok(())
}

struct RawDataDir {
    name_as_number: u32,
    path: path::PathBuf
}

/*
 * Deletes old data
 */
fn clean() -> Result<(), io::Error> {

    // build a list of directories of raw data (that have the right format)
    let dir = env::current_dir().unwrap().join("data").join("raw");

    let mut datasets : Vec<RawDataDir> = vec![];
    let mut latest : u32 = 0; // and keep track of which was latest, as long as you're at it

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        match (&path).file_name() {
            Some(name) => {
                match name.to_str().unwrap().to_string().parse::<u32>() {
                    Ok(number) => {
                        if number > latest {
                            latest = number;
                        }

                        datasets.push(RawDataDir {
                            name_as_number: number,
                            path: path.clone()
                        });
                    },
                    Err(_) => {}
                }
            },
            None => {}
        }
    }

    for entry in datasets {
        if entry.name_as_number == latest {
            continue;
        }

        println!("Clean: removing {}", &entry.path.to_str().unwrap());
        fs::remove_dir_all(&entry.path)?;
    }

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
    let directory = make_data_directory(at)?;

    // generate a list of all the urls you need to download
    let to_download = get_url_queue(at);

    // download them in a threadpool
    let worker_number = cmp::min(DOWNLOAD_WORKERS, to_download.len() as u32);
    let mut handles = vec![];

    let queue = Arc::new(Mutex::new(to_download));

    for _ in 0..worker_number {
        let queue = queue.clone();
        let directory = directory.clone();

        // spawn a worker thread
        let handle = thread::spawn(move || {
            loop {
                // pop urls, or stop the loop if the queue is empty
                // note that it releases the lock as soon as it has the url
                let url = {
                    let mut download_queue = match queue.try_lock() {
                        Ok(locked) => {
                            locked
                        },
                        Err(_) => {
                            continue;
                        }
                    };

                    match download_queue.pop_front() {
                        Some(url) => {
                            url.clone()
                        },
                        None => {
                            break
                        }
                    }
                };

                let filename = (&url).split("/").last().unwrap();
                let to = directory.join(filename);

                let outcome = download_file(&url, &to);

                // clean up if it failed
                match outcome {
                    Ok(_) => {},
                    Err(_) => {
                        // swallow errors
                        match fs::remove_file(&to) {
                            Ok(_) => {},
                            Err(_) => {}
                        }
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
fn make_data_directory(at : DateTime<Utc>) -> Result<path::PathBuf, io::Error> {
    let path = env::current_dir().unwrap().join("data").join("raw").join(at.format("%Y%m%d").to_string());

    fs::create_dir_all(&path)?;

    Ok(path)
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

/*
 * Downloads the file at the given url to the given pathname
 * Returns a result boolean which represents whether it was actually downloaded
 */
fn download_file(url : &String, to : &path::PathBuf) -> Result<bool, PreprocessorError> {

    // if it doesn't exist, then we don't care
    if !dataset_exists(&url)? {
        println!("\t\t Does not exist: {}", &url);
        return Ok(false);
    }

    let bytes = get_content_length(url)?;

    // if it's already downloaded, don't download it again
    if to.exists() {
        let metadata = fs::metadata(to)?;

        // check that it actually downloaded the full thing
        if metadata.len() == bytes {
            println!("\t Already downloaded: {}", &url);
            return Ok(false);
        } else {
            // rename it to what it is, a partial dataset
            fs::rename(to, to.with_extension("grb2.partial"))?;
        }
    }

    // if there's a partially downloaded dataset, remove it
    if to.with_extension("grb2.partial").exists() {
        println!("\t Removing partially downloaded dataset: {}", &url);
        fs::remove_file(to.with_extension("grb2.partial"))?;
    }


    println!("\t Downloading: {} ({} bytes)", &url, bytes);

    Command::new("curl")
        .arg("-o").arg(to.with_extension("grb2.partial"))
        .arg(&url)
        .output()?;

    // check to see if it wrote everything
    let metadata = fs::metadata(to.with_extension("grb2.partial"))?;

    if metadata.len() == bytes {
        fs::rename(to.with_extension("grb2.partial"), to)?;
    } else {
        println!("Only downloaded {} bytes (expecting {}) for {}", metadata.len(), bytes, url)
    }

    Ok(true)
}

/*
 * Gets the content length of the given url, in bytes
 */
fn get_content_length(url : &String) -> Result<u64, PreprocessorError> {
    let output = Command::new("curl")
        .arg("-I")
        .arg(&url)
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.split("\n") {
        let parts : Vec<&str> = line.split(":").collect();

        let header = match parts.get(0) {
            Some(value) => { value }
            None => {
                continue;
            }
        };

        if header != &"Content-Length" {
            continue;
        }

        let value = match parts.get(1) {
            Some(value) => { value }
            None => {
                continue;
            }
        }.trim();

        return match value.parse::<u64>() {
            Ok(content_length) => {
                Ok(content_length)
            },
            Err(_) => {
                Err(PreprocessorError::NoContentLengthError)
            }
        }
    }

    Err(PreprocessorError::NoContentLengthError)
}

