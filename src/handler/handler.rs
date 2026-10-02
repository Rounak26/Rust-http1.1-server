use crate::http::{Request, Response, StatusCode};

pub type Handler = Box<dyn Fn(&Request) -> Response + Send + Sync + 'static>;


pub fn not_found(req: &Request) -> Response {
    Response::text(
        StatusCode::NotFound,
        format!("404 Not Found: {}\n", req.path),
    )
}

pub fn index(_req: &Request) -> Response {
    Response::html(
        StatusCode::Ok,
        "<!doctype html><title>rust-http-server</title><h1>It works</h1>",
    )
}

pub fn health(_req: &Request) -> Response {
    Response::json(StatusCode::Ok, r#"{"status":"ok"}"#)
}

pub fn hello(req: &Request) -> Response {
    let name = req.param("name").unwrap_or("world");
    Response::text(StatusCode::Ok, format!("Hello, {name}!\n"))
}

pub fn echo_query(req: &Request) -> Response {
    let params = req.query_params();
    let mut pairs: Vec<_> = params.iter().collect();
    pairs.sort();
    let body: String = pairs
        .iter()
        .map(|(key, value)| format!("{key} = {value}\n"))
        .collect();
    Response::text(StatusCode::Ok, body)
}

pub fn echo(req: &Request) -> Response {
    Response::text(StatusCode::Ok, req.body.clone())
}
