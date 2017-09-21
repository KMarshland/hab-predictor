use std::io;
use std::convert::From;

use reqwest;

#[derive(Debug)]
pub enum PreprocessorError {
    IOError(io::Error),
    ReqwestError(reqwest::Error),
    NoContentLengthError,
    GenericError
}

impl From<io::Error> for PreprocessorError {
    fn from(error : io::Error) -> Self {
        PreprocessorError::IOError(error)
    }
}

impl From<reqwest::Error> for PreprocessorError {
    fn from(error : reqwest::Error) -> Self {
        PreprocessorError::ReqwestError(error)
    }
}
