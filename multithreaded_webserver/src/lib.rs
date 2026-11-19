//! # Chapter 21: Final Project — Building a Multithreaded Web Server
//!
//! A minimal HTTP/1.1 server using `std::net::TcpListener` and a bounded `ThreadPool`.
//!
//! - [`pool::ThreadPool`]: Fixed-size worker pool pulling boxed closures from a shared MPSC channel.
//! - [`http`]: Minimal HTTP/1.1 request line parsing and response generation.

pub mod http;
pub mod pool;

pub use pool::ThreadPool;
