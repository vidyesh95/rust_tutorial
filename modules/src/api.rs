//! API layer: request-shaped functions that call into the domain.
//!
//! `pub mod handlers;` declares a child module. Because this file is `api.rs`, the child module
//! lives in `src/api/handlers.rs`.

pub mod handlers;

/// Liveness status. Handlers reach this through `super::status`.
#[must_use]
pub fn status() -> &'static str {
    "ok"
}
