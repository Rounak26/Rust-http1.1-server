use std::collections::HashMap;

use crate::error::ParseError;
use crate::http::headers::Headers;
use crate::http::method::Method;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Version {
    Http10,
    Http11,
}

impl Version {
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        match s {
            "HTTP/1.0" => Ok(Version::Http10),
            "HTTP/1.1" => Ok(Version::Http11),
            other => Err(ParseError::InvalidVersion(other.to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Version::Http10 => "HTTP/1.0",
            Version::Http11 => "HTTP/1.1",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub query: Option<String>,
    pub version: Version,
    pub headers: Headers,
    pub body: Vec<u8>,
    pub params: HashMap<String, String>,
}

impl Request {
    pub fn new(method: Method, target: &str) -> Self {
        let (path, query) = match target.split_once('?') {
            Some((p, q)) => (p.to_string(), Some(q.to_string())),
            None => (target.to_string(), None),
        };
        Request {
            method,
            path,
            query,
            version: Version::Http11,
            headers: Headers::new(),
            body: Vec::new(),
            params: HashMap::new(),
        }
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name)
    }

    pub fn param(&self, name: &str) -> Option<&str> {
        self.params.get(name).map(String::as_str)
    }

    pub fn body_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.body).ok()
    }

    pub fn query_params(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        if let Some(query) = &self.query {
            for pair in query.split('&').filter(|p| !p.is_empty()) {
                let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
                map.insert(
                    percent_decode(&key.replace('+', " ")),
                    percent_decode(&value.replace('+', " ")),
                );
            }
        }
        map
    }

    pub fn keep_alive(&self) -> bool {
        let has_token = |token: &str| {
            self.headers
                .get_all("connection")
                .flat_map(|v| v.split(','))
                .any(|t| t.trim().eq_ignore_ascii_case(token))
        };
        match self.version {
            Version::Http11 => !has_token("close"),
            Version::Http10 => has_token("keep-alive"),
        }
    }
}

pub fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
                out.push((hi << 4) | lo);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex_val(b: u8) -> Option<u8> {
    (b as char).to_digit(16).map(|d| d as u8)
}
