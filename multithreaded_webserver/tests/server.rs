//! Integration test (chapter 11): a real TCP socket, a real request, a real response.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use multithreaded_webserver::http::handle_connection;

/// Starts a one-shot server on an ephemeral port, sends `GET {path}`, and returns the raw response.
fn request(path: &str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind an ephemeral port");
    let address = listener.local_addr().expect("read the bound address");

    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("accept one connection");
        handle_connection(stream).expect("write the response");
    });

    let mut client = TcpStream::connect(address).expect("connect to the server");
    let request = format!("GET {path} HTTP/1.1\r\nHost: localhost\r\n\r\n");
    client
        .write_all(request.as_bytes())
        .expect("send the request");

    let mut response = String::new();
    client
        .read_to_string(&mut response)
        .expect("read until the server closes the connection");
    server.join().expect("join the server thread");

    response
}

#[test]
fn serves_the_index_page() {
    let response = request("/");
    assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
    assert!(response.contains("Hello from the Rust book's server"));
}

#[test]
fn serves_health_checks() {
    let response = request("/health");
    assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
    assert!(response.ends_with("ok\n"));
}

#[test]
fn unknown_paths_return_404() {
    let response = request("/nope");
    assert!(response.starts_with("HTTP/1.1 404 Not Found"), "{response}");
    assert!(response.contains("not found"));
}
