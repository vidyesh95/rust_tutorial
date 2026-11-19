//! # Chapter 21: Multithreaded TCP Web Server Binary
//!
//! Run with:
//! ```bash
//! cargo run -p multithreaded_webserver
//! ```
//! Then test with:
//! ```bash
//! curl http://127.0.0.1:7878/
//! curl http://127.0.0.1:7878/health
//! curl http://127.0.0.1:7878/sleep   # Blocks worker for 2 seconds
//! ```

use multithreaded_webserver::{ThreadPool, http};
use std::net::TcpListener;

fn main() {
    let addr = "127.0.0.1:7878";
    let listener = match TcpListener::bind(addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind to {addr}: {e}");
            eprintln!("Note: If port 7878 is in use, verify with `lsof -i :7878`.");
            return;
        }
    };

    println!("=== Chapter 21: Multithreaded Web Server Listening on http://{addr} ===");
    println!("Available endpoints:");
    println!("  GET http://{addr}/        (200 OK HTML response)");
    println!("  GET http://{addr}/health  (200 OK plain text response)");
    println!("  GET http://{addr}/sleep   (Simulates 2s slow request)");
    println!("Server running with a 4-thread worker pool. Press Ctrl+C to terminate.");

    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                pool.execute(|| {
                    if let Err(e) = http::handle_connection(stream) {
                        eprintln!("Error handling connection: {e}");
                    }
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {e}");
            }
        }
    }
}
