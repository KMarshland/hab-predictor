use std::io;
use std::convert::From;

#[derive(Debug)]
pub enum PreprocessorError {
    IOError(io::Error),
    NoContentLengthError
}

impl From<io::Error> for PreprocessorError {
    fn from(error : io::Error) -> Self {
        PreprocessorError::IOError(error)
    }
}
