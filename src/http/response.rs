use crate::http::headers::Headers;
use crate::http::status::StatusCode;

#[derive(Debug, Clone)]
pub struct Response {
    pub status: StatusCode,
    pub headers: Headers,
    pub body: Vec<u8>,
}

impl Response {
    pub fn new(status: StatusCode) -> Self {
        Response {
            status,
            headers: Headers::new(),
            body: Vec::new(),
        }
    }

    pub fn ok() -> Self {
        Self::new(StatusCode::Ok)
    }

    pub fn text(status: StatusCode, body: impl Into<Vec<u8>>) -> Self {
        Self::new(status)
            .with_header("Content-Type", "text/plain; charset=utf-8")
            .with_body(body)
    }

    pub fn html(status: StatusCode, body: impl Into<Vec<u8>>) -> Self {
        Self::new(status)
            .with_header("Content-Type", "text/html; charset=utf-8")
            .with_body(body)
    }

    pub fn json(status: StatusCode, body: impl Into<Vec<u8>>) -> Self {
        Self::new(status)
            .with_header("Content-Type", "application/json")
            .with_body(body)
    }

    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name, value);
        self
    }

    pub fn with_body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }
}
