use std::collections::HashMap;
use std::io::{self, BufRead, Read};

use crate::error::ParseError;
use crate::http::headers::Headers;
use crate::http::method::Method;
use crate::http::request::{Request, Version};

#[derive(Debug, Clone, Copy)]
pub struct Limits {

    pub max_header_bytes: usize,
    pub max_body_bytes: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Limits {
            max_header_bytes: 16 * 1024,
            max_body_bytes: 1024 * 1024,
        }
    }
}

pub fn parse_request<R: BufRead>(reader: &mut R, limits: &Limits) -> Result<Request, ParseError> {
    let mut budget = limits.max_header_bytes;

    let request_line = loop {
        match read_line(reader, &mut budget)? {
            None => return Err(ParseError::ConnectionClosed),
            Some(line) if line.is_empty() => continue,
            Some(line) => break line,
        }
    };
    let (method, path, query, version) = parse_request_line(&request_line)?;

    let mut headers = Headers::new();
    loop {
        match read_line(reader, &mut budget)? {
            None => return Err(io::Error::from(io::ErrorKind::UnexpectedEof).into()),
            Some(line) if line.is_empty() => break,
            Some(line) => {
                let (name, value) = parse_header_line(&line)?;
                headers.append(name, value);
            }
        }
    }

    if version == Version::Http11 && !headers.contains("host") {
        return Err(ParseError::MissingHost);
    }

    if headers.contains("transfer-encoding") {
        return Err(ParseError::UnsupportedTransferEncoding);
    }

    let body_len = parse_content_length(&headers)?;
    if body_len > limits.max_body_bytes {
        return Err(ParseError::BodyTooLarge);
    }
    let mut body = vec![0u8; body_len];
    reader.read_exact(&mut body)?;

    Ok(Request {
        method,
        path,
        query,
        version,
        headers,
        body,
        params: HashMap::new(),
    })
}

fn read_line<R: BufRead>(reader: &mut R, budget: &mut usize) -> Result<Option<String>, ParseError> {
    if *budget == 0 {
        return Err(ParseError::HeadersTooLarge);
    }

    let mut buf = Vec::new();
    let n = reader
        .by_ref()
        .take(*budget as u64)
        .read_until(b'\n', &mut buf)?;
    if n == 0 {
        return Ok(None);
    }
    *budget -= n;

    if buf.last() != Some(&b'\n') {

        return Err(if *budget == 0 {
            ParseError::HeadersTooLarge
        } else {
            io::Error::from(io::ErrorKind::UnexpectedEof).into()
        });
    }

    buf.pop();
    if buf.last() == Some(&b'\r') {
        buf.pop();
    }
    String::from_utf8(buf)
        .map(Some)
        .map_err(|_| ParseError::InvalidEncoding)
}

type RequestLine = (Method, String, Option<String>, Version);

fn parse_request_line(line: &str) -> Result<RequestLine, ParseError> {
    let mut parts = line.split(' ');
    let (method, target, version) = match (parts.next(), parts.next(), parts.next(), parts.next())
    {
        (Some(m), Some(t), Some(v), None) => (m, t, v),
        _ => return Err(ParseError::MalformedRequestLine),
    };

    let method = method.parse::<Method>()?;
    let version = Version::parse(version)?;

    // Only origin-form targets ("/path?query") and "*" are supported.
    if !(target.starts_with('/') || target == "*") {
        return Err(ParseError::MalformedRequestLine);
    }
    let (path, query) = match target.split_once('?') {
        Some((p, q)) => (p.to_string(), Some(q.to_string())),
        None => (target.to_string(), None),
    };

    Ok((method, path, query, version))
}

fn parse_header_line(line: &str) -> Result<(&str, &str), ParseError> {
    let (name, value) = line.split_once(':').ok_or(ParseError::MalformedHeader)?;

    // The name must be a non-empty token. This also rejects obsolete line folding
    // (a line starting with whitespace) and whitespace before the colon.
    if name.is_empty() || !name.bytes().all(is_token_byte) {
        return Err(ParseError::MalformedHeader);
    }
    Ok((name, value.trim()))
}

fn is_token_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&b)
}

fn parse_content_length(headers: &Headers) -> Result<usize, ParseError> {
    let mut length: Option<usize> = None;
    for value in headers.get_all("content-length") {
        // `usize::from_str` accepts a leading '+', so check the digits ourselves.
        if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
            return Err(ParseError::InvalidContentLength);
        }
        let parsed = value
            .parse::<usize>()
            .map_err(|_| ParseError::InvalidContentLength)?;
        match length {
            Some(previous) if previous != parsed => return Err(ParseError::InvalidContentLength),
            _ => length = Some(parsed),
        }
    }
    Ok(length.unwrap_or(0))
}
