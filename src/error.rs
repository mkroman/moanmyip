use std::io::Error as IoError;

#[derive(Debug, Fail)]
pub enum Error {
    /// An error occurred related to HTTP.
    #[fail(display = "HTTP error")]
    ReqwestError {
        #[fail(cause)]
        error: reqwest::Error,
    },
    #[fail(display = "URL parse error")]
    UrlParseError {
        #[fail(cause)]
        error: url::ParseError,
    },
    /// There was an I/O error.
    #[fail(display = "I/O error: {}", error)]
    IoError {
        #[fail(cause)]
        error: IoError,
    },
    #[fail(display = "external IP address could not be extracted")]
    ExternalIpMissingError,
    #[fail(display = "audio clip could not be extracted")]
    AudioClipSrcMissingError,
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Error {
        Error::IoError { error }
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Error {
        Error::ReqwestError { error }
    }
}

impl From<url::ParseError> for Error {
    fn from(error: url::ParseError) -> Error {
        Error::UrlParseError { error }
    }
}
