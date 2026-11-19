//! Handlers: the seam between "a request arrived" and "the domain did something".
//!
//! `crate::` starts at the library root; `super::` starts at the parent module (`api`).

use super::status;
use crate::domain::{DomainError, User};

/// Handles `POST /users` (simplified: `raw_name` arrives as a query parameter).
///
/// # Errors
///
/// Returns an HTTP-ready error string when the domain rejects the input.
pub fn create_user(id: u64, raw_name: &str) -> Result<User, String> {
    User::new(id, raw_name).map_err(describe_error)
}

/// Routes a path to a response body. A stand-in for a real HTTP router.
#[must_use]
pub fn route(path: &str) -> String {
    match path {
        "/health" => status().to_owned(),
        _ => "404 Not Found".to_owned(),
    }
}

fn describe_error(error: DomainError) -> String {
    match error {
        DomainError::EmptyName => "400 Bad Request: name must not be empty".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_route_uses_the_parent_module() {
        assert_eq!(route("/health"), "ok");
    }

    #[test]
    fn unknown_routes_return_404() {
        assert_eq!(route("/nope"), "404 Not Found");
    }
}
