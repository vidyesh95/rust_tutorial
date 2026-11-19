//! Minimal HTTP/1.1 parsing and response building.
//!
//! Deliberately small: read one request, find the path, write one response with `Connection: close`.
//! Production servers add keep-alive, chunked bodies, timeouts, header parsing, and TLS.

use std::io::{Read, Write};
use std::net::TcpStream;

/// The three space-separated parts of an HTTP request line.
#[derive(Debug, PartialEq, Eq)]
pub struct RequestLine<'a> {
    /// `GET`, `POST`, ...
    pub method: &'a str,
    /// `/`, `/health`, ...
    pub path: &'a str,
    /// `HTTP/1.1`
    pub version: &'a str,
}

impl<'a> RequestLine<'a> {
    /// Parses `GET /path HTTP/1.1`.
    ///
    /// Returns `None` unless the line has exactly three whitespace-separated parts.
    #[must_use]
    pub fn parse(line: &'a str) -> Option<Self> {
        let mut parts = line.split_whitespace();
        let method = parts.next()?;
        let path = parts.next()?;
        let version = parts.next()?;
        if parts.next().is_some() {
            return None;
        }
        Some(Self {
            method,
            path,
            version,
        })
    }
}

/// Builds a complete HTTP response with a byte-accurate `Content-Length`.
#[must_use]
pub fn response(status: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {status}\r\nContent-Length: {}\r\nContent-Type: text/plain; charset=utf-8\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

/// Maps a path to a response — the "router".
#[must_use]
pub fn response_for(path: &str) -> String {
    match path {
        "/" | "/index.html" => response("200 OK", "Hello from the Rust book's server\n"),
        "/health" => response("200 OK", "ok\n"),
        "/sleep" => {
            std::thread::sleep(std::time::Duration::from_secs(2));
            response("200 OK", "Awake after 2 seconds!\n")
        }
        _ => response("404 Not Found", "not found\n"),
    }
}

/// Reads one request from a connection and writes one response.
///
/// # Errors
///
/// Propagates socket read and write failures.
pub fn handle_connection(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buffer = [0_u8; 1024];
    let bytes_read = stream.read(&mut buffer)?;
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);

    let path = request
        .lines()
        .next()
        .and_then(RequestLine::parse)
        .map_or("/", |request| request.path);

    stream.write_all(response_for(path).as_bytes())?;
    stream.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_request_line() {
        let line = RequestLine::parse("GET /health HTTP/1.1").expect("valid request line");
        assert_eq!(line.method, "GET");
        assert_eq!(line.path, "/health");
        assert_eq!(line.version, "HTTP/1.1");
    }

    #[test]
    fn rejects_malformed_request_lines() {
        assert_eq!(RequestLine::parse("GET"), None);
        assert_eq!(RequestLine::parse("GET / HTTP/1.1 extra"), None);
    }

    #[test]
    fn routes_known_paths_and_unknown_paths() {
        assert!(response_for("/").starts_with("HTTP/1.1 200 OK"));
        assert!(response_for("/health").contains("ok"));
        assert!(response_for("/missing").starts_with("HTTP/1.1 404 Not Found"));
    }

    #[test]
    fn content_length_matches_the_body() {
        let response = response("200 OK", "hello");
        assert!(response.contains("Content-Length: 5"));
    }
}
