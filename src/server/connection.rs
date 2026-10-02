use std::io::{self, BufReader, Write};
use std::net::{Shutdown, TcpStream};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

use crate::config::Config;
use crate::error::ParseError;
use crate::http::parser::{parse_request, Limits};
use crate::http::serializer::serialize;
use crate::http::{Method, Request, Response, StatusCode};
use crate::router::Router;

pub fn handle(stream: TcpStream, router: Arc<Router>, config: Arc<Config>) {
    let peer = stream
        .peer_addr()
        .map(|addr| addr.to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    if let Err(err) = serve(stream, &router, &config) {
        eprintln!("[{peer}] connection error: {err}");
    }
}

fn serve(stream: TcpStream, router: &Router, config: &Config) -> io::Result<()> {
    stream.set_read_timeout(Some(config.read_timeout))?;
    stream.set_write_timeout(Some(config.write_timeout))?;
    stream.set_nodelay(true)?;

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut writer = stream;
    let limits = Limits {
        max_header_bytes: config.max_header_bytes,
        max_body_bytes: config.max_body_bytes,
    };

    let mut served = 0;
    while served < config.max_requests_per_connection {
        served += 1;

        let request = match parse_request(&mut reader, &limits) {
            Ok(request) => request,
            
            Err(ParseError::ConnectionClosed) => break,
            Err(ParseError::Io(err)) if is_timeout(&err) => break,
            Err(ParseError::Io(err)) => return Err(err),
            Err(err) => {
                let response = Response::text(err.status(), format!("{err}\n"))
                    .with_header("Connection", "close");
                writer.write_all(&serialize(&response, false))?;
                break;
            }
        };

        let is_last = served == config.max_requests_per_connection;
        let keep_alive = request.keep_alive() && !is_last;
        let head_only = request.method == Method::Head;
        let (method, path) = (request.method, request.path.clone());

        let mut response = dispatch(router, request);
        response.headers.insert(
            "Connection",
            if keep_alive { "keep-alive" } else { "close" },
        );
        println!("{method} {path} -> {}", response.status.code());

        writer.write_all(&serialize(&response, head_only))?;
        if !keep_alive {
            break;
        }
    }

    let _ = writer.shutdown(Shutdown::Both);
    Ok(())
}

fn dispatch(router: &Router, request: Request) -> Response {
    catch_unwind(AssertUnwindSafe(|| router.dispatch(request))).unwrap_or_else(|_| {
        Response::text(StatusCode::InternalServerError, "500 Internal Server Error\n")
    })
}

fn is_timeout(err: &io::Error) -> bool {
    matches!(
        err.kind(),
        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
    )
}
