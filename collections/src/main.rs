//! # Chapter 8: Common Collections (`Vec<T>`, `String`, `HashMap<K, V>`)
//!
//! ## Key Backend Insights:
//! - **`Vec::with_capacity(n)`**: When receiving a batch of records from a database or JSON payload,
//!   pre-allocating vector capacity prevents repeated heap reallocations and memory copies.
//! - **UTF-8 Strings**: Rust prevents indexing into strings (`s[0]`) because UTF-8 characters can be
//!   1 to 4 bytes long. Indexing by byte could split a multibyte character and corrupt memory.
//! - **HashMap Entry API (`entry().or_insert()`)**: Highly idiomatic pattern for in-memory caching,
//!   rate-limiting counters, and session stores.

use std::collections::HashMap;

// --- 1. Vectors & Pre-allocation ---

pub fn vector_demo() {
    println!("\n=== 1. Vectors (Vec<T>) ===");

    // Basic vector
    let mut numbers = vec![10, 20, 30];
    numbers.push(40);

    // Safe access using .get() vs unsafe direct indexing []
    // Direct indexing `numbers[99]` will PANIC and crash the server thread!
    // .get() returns Option<&T>, allowing safe error handling.
    match numbers.get(2) {
        Some(val) => println!("Element at index 2: {val}"),
        None => println!("Index out of bounds!"),
    }

    match numbers.get(99) {
        Some(val) => println!("Element at index 99: {val}"),
        None => println!("Index 99 is safely identified as out of bounds."),
    }

    // Backend Best Practice: Pre-allocating capacity for batch processing
    let batch_size = 1000;
    let mut preallocated: Vec<u64> = Vec::with_capacity(batch_size);
    println!(
        "Preallocated capacity: {}, initial len: {}",
        preallocated.capacity(),
        preallocated.len()
    );
    for i in 0..batch_size as u64 {
        preallocated.push(i * 2);
    }
    println!(
        "After fill: capacity: {}, len: {}",
        preallocated.capacity(),
        preallocated.len()
    );
}

// --- 2. Strings & UTF-8 ---

pub fn strings_demo() {
    println!("\n=== 2. UTF-8 Strings ===");

    let mut message = String::from("Hello");
    message.push_str(", Rustaceans! 🦀");
    println!("Message: {message}");
    println!(
        "Byte length (note the crab emoji is 4 bytes): {}",
        message.len()
    );

    // Iterating over characters safely:
    let char_count = message.chars().count();
    println!("Unicode character count: {char_count}");
}

// --- 3. HashMaps & The Entry API (Backend In-Memory Cache) ---

/// In-memory metric store tracking hit counts per API route.
#[derive(Debug, Default)]
pub struct RouteMetrics {
    endpoint_hits: HashMap<String, u64>,
}

impl RouteMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records an access hit on a given route using the Entry API.
    pub fn record_hit(&mut self, endpoint: &str) -> u64 {
        // `entry(k).and_modify(f).or_insert(v)` is atomic and eliminates duplicate lookups
        let count = self
            .endpoint_hits
            .entry(endpoint.to_string())
            .and_modify(|c| *c += 1)
            .or_insert(1);
        *count
    }

    /// Retrieves total hits for an endpoint, returning 0 if not found.
    pub fn get_hits(&self, endpoint: &str) -> u64 {
        self.endpoint_hits.get(endpoint).copied().unwrap_or(0)
    }

    /// Returns a list of all endpoints and their hit counts sorted by descending traffic.
    pub fn top_endpoints(&self) -> Vec<(String, u64)> {
        let mut list: Vec<(String, u64)> = self
            .endpoint_hits
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        list.sort_by_key(|entry| std::cmp::Reverse(entry.1));
        list
    }
}

fn main() {
    println!("=== Chapter 8: Collections in Action ===");
    vector_demo();
    strings_demo();

    println!("\n=== 3. Backend Route Metrics via HashMap ===");
    let mut metrics = RouteMetrics::new();
    metrics.record_hit("/api/v1/users");
    metrics.record_hit("/api/v1/users");
    metrics.record_hit("/api/v1/products");
    metrics.record_hit("/api/v1/users");
    metrics.record_hit("/health");

    println!(
        "Hits for /api/v1/users:    {}",
        metrics.get_hits("/api/v1/users")
    );
    println!(
        "Hits for /api/v1/products: {}",
        metrics.get_hits("/api/v1/products")
    );
    println!(
        "Hits for /unknown:         {}",
        metrics.get_hits("/unknown")
    );

    println!("\nTop endpoints ranking:");
    for (route, count) in metrics.top_endpoints() {
        println!("  {count:3} hits -> {route}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_safe_get() {
        let mut items = vec!["alpha", "beta"];
        items.push("gamma");
        assert_eq!(items.get(1), Some(&"beta"));
        assert_eq!(items.first(), Some(&"alpha"));
        assert_eq!(items.get(5), None);
    }

    #[test]
    fn test_route_metrics() {
        let mut metrics = RouteMetrics::new();
        assert_eq!(metrics.record_hit("/login"), 1);
        assert_eq!(metrics.record_hit("/login"), 2);
        assert_eq!(metrics.record_hit("/logout"), 1);

        assert_eq!(metrics.get_hits("/login"), 2);
        assert_eq!(metrics.get_hits("/logout"), 1);
        assert_eq!(metrics.get_hits("/nonexistent"), 0);

        let top = metrics.top_endpoints();
        assert_eq!(top[0], ("/login".to_string(), 2));
        assert_eq!(top[1], ("/logout".to_string(), 1));
    }
}
