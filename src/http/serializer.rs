use crate::http::response::Response;

pub fn serialize(response: &Response, head_only: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(160 + response.body.len());

    out.extend_from_slice(
        format!(
            "HTTP/1.1 {} {}\r\n",
            response.status.code(),
            response.status.reason()
        )
        .as_bytes(),
    );

    let has_line_break = |s: &str| s.contains(|c| c == '\r' || c == '\n');

    for (name, value) in response.headers.iter() {
        if name.eq_ignore_ascii_case("content-length")
            || name.is_empty()
            || has_line_break(name)
            || has_line_break(value)
        {
            continue;
        }
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(b": ");
        out.extend_from_slice(value.as_bytes());
        out.extend_from_slice(b"\r\n");
    }

    if !response.headers.contains("server") {
        out.extend_from_slice(
            concat!("Server: rust-http-server/", env!("CARGO_PKG_VERSION"), "\r\n").as_bytes(),
        );
    }

    let has_body = response.status.allows_body();
    if has_body {
        out.extend_from_slice(format!("Content-Length: {}\r\n", response.body.len()).as_bytes());
    }
    out.extend_from_slice(b"\r\n");

    if has_body && !head_only {
        out.extend_from_slice(&response.body);
    }
    out
}
