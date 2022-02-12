//! HTTP Versions enum
//!
//! Instead of relying on typo-prone Strings, use expected HTTP versions as
//! the `HttpVersion` enum.
use std::fmt;
use std::str::FromStr;

use crate::error::Error;
use self::HttpVersion::{Http09, Http10, Http11, Http20};

/// Represents a version of the HTTP spec.
#[derive(PartialEq, PartialOrd, Copy, Clone, Eq, Ord, Hash, Debug)]
pub enum HttpVersion {
    /// `HTTP/0.9`
    Http09,
    /// `HTTP/1.0`
    Http10,
    /// `HTTP/1.1`
    Http11,
    /// `HTTP/2.0`
    Http20,
}

impl From<HttpVersion> for http::Version {
    fn from(arg: HttpVersion) -> Self {
        match arg{
            Http09 => {http::Version::HTTP_09}
            Http10 => {http::Version::HTTP_10}
            Http11 => {http::Version::HTTP_11}
            Http20 => {http::Version::HTTP_2}
        }
    }
}


impl fmt::Display for HttpVersion {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(self.as_ref())
    }
}

impl AsRef<str> for HttpVersion {
    fn as_ref(&self) -> &str {
        match *self {
            Http09 => "HTTP/0.9",
            Http10 => "HTTP/1.0",
            Http11 => "HTTP/1.1",
            Http20 => "HTTP/2.0",
        }
    }
}

impl FromStr for HttpVersion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HTTP/0.9" => Ok(Http09),
            "HTTP/1.0" => Ok(Http10),
            "HTTP/1.1" => Ok(Http11),
            "HTTP/2.0" => Ok(Http20),
            _ => Err(Error::Version),
        }
    }
}
