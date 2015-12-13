use std::error;
use std::fmt;
use std::io;
use std::num;

#[derive(Debug)]
pub enum Error {
    FormatError,
    Other(Box<error::Error>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::FormatError => write!(f, "Invalid format for vocabulary file"),
            Error::Other(ref e) => e.fmt(f),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::FormatError => "Invalid format for vocabulary file",
            Error::Other(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::FormatError => None,
            Error::Other(ref e) => e.cause(),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Other(Box::new(e))
    }
}

impl From<num::ParseIntError> for Error {
    fn from(e: num::ParseIntError) -> Self {
        Error::Other(Box::new(e))
    }
}
