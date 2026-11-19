//! Chapter 7 — Packages, Crates, and Modules.
//!
//! A **package** is a `Cargo.toml`; it can contain a library crate and several binary crates. This
//! package has both: `src/lib.rs` is the library root, `src/main.rs` is a binary that depends on it.
//!
//! - `pub mod` exposes a module to other crates.
//! - `mod` keeps a module private to this crate.
//! - `pub use` re-exports an item so callers can reach it through the crate root.
//!
//! ```
//! use modules::User;
//!
//! let user = User::new(1, " Ada ").expect("valid user");
//! assert_eq!(user.name(), "ada");
//! ```
//!
//! Run `cargo run -p modules` for the binary and `cargo test -p modules` for unit, integration, and
//! doc tests.

pub mod api;
pub mod domain;

mod internal;

// Re-export: callers can write `modules::User` instead of `modules::domain::User`.
pub use domain::User;
