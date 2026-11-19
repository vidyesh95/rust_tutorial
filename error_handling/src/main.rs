//! # Chapter 9: Error Handling (`Result<T, E>`, `?`, and API Errors)
//!
//! ## Key Backend Insights:
//! - **Never `unwrap()` in Request Handlers**: An unhandled `unwrap()` panics the worker thread,
//!   abruptly terminates the HTTP connection, and degrades service reliability.
//! - **The `?` Operator**: Propagates errors cleanly up to the top-level route handler.
//! - **Domain Error Mapping**: In backend services, all lower-level errors (database query errors,
//!   JSON parsing errors, validation failures) are converted into a typed `ApiError`
//!   which cleanly translates to an HTTP status code (400, 401, 404, 500) and structured response.

use std::error::Error;
use std::fmt;

// --- 1. Custom Domain Error Enum ---

#[derive(Debug, PartialEq, Eq)]
pub enum ApiError {
    BadRequest(String),
    Unauthorized(String),
    NotFound { resource: String, id: String },
    DatabaseError(String),
    InternalServerError(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::BadRequest(msg) => write!(f, "Bad Request: {msg}"),
            ApiError::Unauthorized(msg) => write!(f, "Unauthorized: {msg}"),
            ApiError::NotFound { resource, id } => {
                write!(f, "{resource} with ID '{id}' was not found")
            }
            ApiError::DatabaseError(msg) => write!(f, "Database failure: {msg}"),
            ApiError::InternalServerError(msg) => write!(f, "Internal error: {msg}"),
        }
    }
}

impl Error for ApiError {}

impl ApiError {
    /// Maps each domain error to its corresponding HTTP Status Code.
    pub fn http_status_code(&self) -> u16 {
        match self {
            ApiError::BadRequest(_) => 400,
            ApiError::Unauthorized(_) => 401,
            ApiError::NotFound { .. } => 404,
            ApiError::DatabaseError(_) => 500,
            ApiError::InternalServerError(_) => 500,
        }
    }

    /// Formats the error as a JSON-compatible error payload.
    pub fn to_json_response(&self) -> String {
        let code = self.http_status_code();
        let message = self.to_string();
        format!(r#"{{"status": {code}, "error": "{message}"}}"#)
    }
}

// --- 2. Propagating Errors with the `?` Operator ---

/// Simulated user record.
#[derive(Debug, PartialEq, Eq)]
pub struct UserProfile {
    pub id: u64,
    pub name: String,
    pub age: u8,
}

/// Simulated database query that can fail.
fn mock_db_query(user_id: u64) -> Result<Option<(String, String)>, ApiError> {
    match user_id {
        0 => Err(ApiError::DatabaseError(
            "Connection pool timeout".to_string(),
        )),
        1 => Ok(Some(("Alice".to_string(), "28".to_string()))),
        2 => Ok(Some(("Bob".to_string(), "invalid_age_str".to_string()))),
        _ => Ok(None),
    }
}

/// Service function demonstrating `?`, `.ok_or()`, and `.map_err()`.
pub fn get_user_profile(user_id_str: &str) -> Result<UserProfile, ApiError> {
    // 1. Parse string ID to u64, mapping ParseIntError -> ApiError::BadRequest
    let user_id = user_id_str
        .parse::<u64>()
        .map_err(|_| ApiError::BadRequest(format!("'{user_id_str}' is not a valid integer ID")))?;

    // 2. Query database using `?` for propagation
    let raw_record = mock_db_query(user_id)?;

    // 3. Convert Option<Record> -> Result<Record, ApiError::NotFound> using .ok_or()
    let (name, raw_age) = raw_record.ok_or_else(|| ApiError::NotFound {
        resource: "User".to_string(),
        id: user_id.to_string(),
    })?;

    // 4. Parse age string to u8
    let age = raw_age
        .parse::<u8>()
        .map_err(|e| ApiError::InternalServerError(format!("Corrupt user age data: {e}")))?;

    Ok(UserProfile {
        id: user_id,
        name,
        age,
    })
}

// --- 3. Simulated Controller Handler ---

fn handle_get_user_request(raw_id: &str) -> (u16, String) {
    match get_user_profile(raw_id) {
        Ok(user) => (
            200,
            format!(
                r#"{{"status": 200, "data": {{"id": {}, "name": "{}"}}}}"#,
                user.id, user.name
            ),
        ),
        Err(err) => (err.http_status_code(), err.to_json_response()),
    }
}

fn main() {
    println!("=== Chapter 9: Error Handling in Backend Services ===");

    let test_ids = [
        ("1", "Valid user Alice"),
        ("abc", "Invalid non-numeric ID"),
        ("99", "Non-existent user ID"),
        ("0", "Database timeout error"),
        ("2", "Corrupt database data"),
    ];

    for (id, description) in test_ids {
        println!("\nRequest: GET /users/{id} ({description})");
        let (status, body) = handle_get_user_request(id);
        println!("  HTTP Status: {status}");
        println!("  HTTP Body:   {body}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_user() {
        let profile = get_user_profile("1").expect("Should find user 1");
        assert_eq!(
            profile,
            UserProfile {
                id: 1,
                name: "Alice".to_string(),
                age: 28
            }
        );
    }

    #[test]
    fn test_invalid_id() {
        let err = get_user_profile("not_a_number").unwrap_err();
        assert_eq!(
            err,
            ApiError::BadRequest("'not_a_number' is not a valid integer ID".to_string())
        );
        assert_eq!(err.http_status_code(), 400);
    }

    #[test]
    fn test_not_found() {
        let err = get_user_profile("42").unwrap_err();
        assert_eq!(
            err,
            ApiError::NotFound {
                resource: "User".to_string(),
                id: "42".to_string()
            }
        );
        assert_eq!(err.http_status_code(), 404);
    }

    #[test]
    fn test_db_failure() {
        let err = get_user_profile("0").unwrap_err();
        assert_eq!(
            err,
            ApiError::DatabaseError("Connection pool timeout".to_string())
        );
        assert_eq!(err.http_status_code(), 500);
    }
}
