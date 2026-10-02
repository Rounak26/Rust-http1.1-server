use std::collections::HashMap;

use crate::handler::Handler;
use crate::http::request::percent_decode;
use crate::http::{Method, Request, Response};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Segment {
    Literal(String),
    Param(String),
}

pub struct Route {
    pub method: Method,
    segments: Vec<Segment>,
    handler: Handler,
}

impl Route {
    pub fn new(method: Method, pattern: &str, handler: Handler) -> Self {
        let segments = split_path(pattern)
            .map(|segment| match segment.strip_prefix(':') {
                Some(name) => Segment::Param(name.to_string()),
                None => Segment::Literal(segment.to_string()),
            })
            .collect();
        Route {
            method,
            segments,
            handler,
        }
    }

    pub fn matches(&self, path: &str) -> Option<HashMap<String, String>> {
        let mut params = HashMap::new();
        let mut parts = split_path(path);

        for segment in &self.segments {
            let part = parts.next()?;
            match segment {
                Segment::Literal(expected) if expected == part => {}
                Segment::Literal(_) => return None,
                Segment::Param(name) => {
                    params.insert(name.clone(), percent_decode(part));
                }
            }
        }

        if parts.next().is_some() {
            return None;
        }
        Some(params)
    }

    pub fn call(&self, request: &Request) -> Response {
        (self.handler)(request)
    }
}

fn split_path(path: &str) -> impl Iterator<Item = &str> {
    path.split('/').filter(|s| !s.is_empty())
}
