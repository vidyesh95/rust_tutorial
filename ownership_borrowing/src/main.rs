//! # Chapter 4: Ownership, References, Borrowing, and Slices
//!
//! ## Key Backend Insights:
//! - **Move Semantics**: Prevent double-free errors. When passing a request payload to a handler,
//!   decide whether the handler takes ownership (`User`) or only reads it (`&User`).
//! - **The Borrowing Rule**: Either *any number of immutable references (`&T`)* OR *exactly one mutable reference (`&mut T`)*.
//!   This completely prevents data races across threads and memory corruption at compile time.
//! - **Slices (`&str`, `&[T]`)**: Enable **zero-copy** parsing of HTTP headers, request lines, and binary payloads.

/// Demonstrates move vs copy semantics.
fn move_vs_copy_demo() {
    println!("\n=== 1. Move vs Copy Semantics ===");

    // Scalar types stored on the stack implement the `Copy` trait.
    let x = 42;
    let y = x; // Copy: x is still valid!
    println!("Stack copy: x = {x}, y = {y}");

    // Heap-allocated types (String, Vec) do NOT implement `Copy`.
    let original = String::from("Authorization: Bearer token_xyz");
    let moved = original; // Move: ownership transferred to `moved`.
    // println!("{original}"); // COMPILE ERROR: value borrowed here after move!
    println!("Heap move: moved string = '{moved}'");

    // To duplicate heap data, explicitly call .clone()
    let cloned = moved.clone();
    println!("Cloned: moved = '{moved}', cloned = '{cloned}'");
}

/// Demonstrates immutable borrowing (`&T` / `&str`).
/// In backend handlers, pass string slices (`&str`) instead of `&String`.
/// Rust's deref coercion allows passing both `&String` and string literals `&str`.
fn calculate_header_length(header: &str) -> usize {
    header.len() // Read-only access; `header` is not dropped when this function exits.
}

/// Demonstrates mutable borrowing (`&mut T`).
/// Allows modifying data in-place without re-allocating or returning new copies.
fn append_query_param(url: &mut String, key: &str, value: &str) {
    if !url.contains('?') {
        url.push('?');
    } else {
        url.push('&');
    }
    url.push_str(key);
    url.push('=');
    url.push_str(value);
}

fn borrowing_demo() {
    println!("\n=== 2. Borrowing & References ===");

    let request_id = String::from("req_987654321");
    // Passing &request_id borrows it without moving ownership
    let len = calculate_header_length(&request_id);
    println!("Request ID '{request_id}' has length {len} bytes.");

    let mut endpoint = String::from("/api/v1/products");
    println!("Initial endpoint: {endpoint}");

    // Mutable borrow
    append_query_param(&mut endpoint, "category", "electronics");
    append_query_param(&mut endpoint, "page", "1");
    println!("Updated endpoint with query params: {endpoint}");

    // Borrow checker rule demonstration:
    // Multiple readers are allowed simultaneously:
    let r1 = &endpoint;
    let r2 = &endpoint;
    println!("Readers can coexist: r1='{r1}', r2='{r2}'");
    // Once readers are no longer used (Non-Lexical Lifetimes), a writer can be created:
    let w = &mut endpoint;
    w.push_str("&sort=asc");
    println!("Writer modified endpoint: {w}");
}

/// Zero-copy HTTP Request Line Parser using Slices (`&str`).
///
/// In languages like Python, `line.split(' ')` creates 3 new heap-allocated strings.
/// In Rust, slices (`&str`) point directly into the original string's memory,
/// allocating ZERO new heap memory!
#[derive(Debug, PartialEq, Eq)]
pub struct HttpRequestLine<'a> {
    pub method: &'a str,
    pub path: &'a str,
    pub protocol: &'a str,
}

pub fn parse_http_request_line(raw_line: &str) -> Option<HttpRequestLine<'_>> {
    let mut parts = raw_line.split_whitespace();
    let method = parts.next()?;
    let path = parts.next()?;
    let protocol = parts.next()?;

    Some(HttpRequestLine {
        method,
        path,
        protocol,
    })
}

fn slices_demo() {
    println!("\n=== 3. Slices (&str and &[T]) & Zero-Copy Parsing ===");

    let raw_request = "GET /api/v2/users/42 HTTP/1.1";
    if let Some(parsed) = parse_http_request_line(raw_request) {
        println!("Successfully parsed HTTP request line without any heap allocations:");
        println!("  Method:   {}", parsed.method);
        println!("  Path:     {}", parsed.path);
        println!("  Protocol: {}", parsed.protocol);
    }

    // Array slices
    let status_codes = [200, 201, 204, 400, 401, 403, 404, 500];
    let success_codes: &[i32] = &status_codes[0..3];
    let error_codes: &[i32] = &status_codes[3..];
    println!("Success status codes slice: {success_codes:?}");
    println!("Error status codes slice:   {error_codes:?}");
}

fn main() {
    println!("=== Chapter 4: Ownership, Borrowing & Slices in Action ===");
    move_vs_copy_demo();
    borrowing_demo();
    slices_demo();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_header_length() {
        let header = String::from("Content-Type: application/json");
        assert_eq!(calculate_header_length(&header), 30);
    }

    #[test]
    fn test_append_query_param() {
        let mut url = String::from("/search");
        append_query_param(&mut url, "q", "rust");
        append_query_param(&mut url, "limit", "10");
        assert_eq!(url, "/search?q=rust&limit=10");
    }

    #[test]
    fn test_parse_http_request_line() {
        let request = "POST /api/login HTTP/2.0";
        let parsed = parse_http_request_line(request).expect("Parsing should succeed");
        assert_eq!(
            parsed,
            HttpRequestLine {
                method: "POST",
                path: "/api/login",
                protocol: "HTTP/2.0"
            }
        );
    }

    #[test]
    fn test_parse_invalid_request_line() {
        let invalid = "INVALID_REQUEST";
        assert_eq!(parse_http_request_line(invalid), None);
    }
}
