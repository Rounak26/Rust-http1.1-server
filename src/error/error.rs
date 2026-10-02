use std::fmt;
use std::io;

use crate::http::status::StatusCode;

pub type Result<T> = std::result::Result<T, ServerError>;

#[derive(Debug)]
pub enum ServerError {
    Io(io::Error),
    Config(String),
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::Io(e) => write!(f, "I/O error: {e}"),
            ServerError::Config(msg) => write!(f, "configuration error: {msg}"),
        }
    }
}

impl std::error::Error for ServerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ServerError::Io(e) => Some(e),
            ServerError::Config(_) => None,
        }
    }
}

impl From<io::Error> for ServerError {
    fn from(e: io::Error) -> Self {
        ServerError::Io(e)
    }
}

#[derive(Debug)]
pub enum ParseError {
    ConnectionClosed,
    Io(io::Error),
    InvalidEncoding,
    MalformedRequestLine,
    InvalidMethod(String),
    InvalidVersion(String),
    MalformedHeader,
    MissingHost,
    HeadersTooLarge,
    InvalidContentLength,
    BodyTooLarge,
    UnsupportedTransferEncoding,
}

impl ParseError {
    pub fn status(&self) -> StatusCode {
        match self {
            ParseError::InvalidMethod(_) => StatusCode::NotImplemented,
            ParseError::InvalidVersion(_) => StatusCode::HttpVersionNotSupported,
            ParseError::HeadersTooLarge => StatusCode::RequestHeaderFieldsTooLarge,
            ParseError::BodyTooLarge => StatusCode::PayloadTooLarge,
            ParseError::UnsupportedTransferEncoding => StatusCode::NotImplemented,
            _ => StatusCode::BadRequest,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::ConnectionClosed => write!(f, "connection closed"),
            ParseError::Io(e) => write!(f, "I/O error: {e}"),
            ParseError::InvalidEncoding => write!(f, "request head is not valid UTF-8"),
            ParseError::MalformedRequestLine => write!(f, "malformed request line"),
            ParseError::InvalidMethod(_) => write!(f, "unsupported method"),
            ParseError::InvalidVersion(_) => write!(f, "unsupported HTTP version"),
            ParseError::MalformedHeader => write!(f, "malformed header"),
            ParseError::MissingHost => write!(f, "missing Host header"),
            ParseError::HeadersTooLarge => write!(f, "request head too large"),
            ParseError::InvalidContentLength => write!(f, "invalid Content-Length"),
            ParseError::BodyTooLarge => write!(f, "request body too large"),
            ParseError::UnsupportedTransferEncoding => {
                write!(f, "Transfer-Encoding is not supported")
            }
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for ParseError {
    fn from(e: io::Error) -> Self {
        ParseError::Io(e)
    }
}
