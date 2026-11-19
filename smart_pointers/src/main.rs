//! # Chapter 15: Smart Pointers (`Box`, `Deref`, `Drop`, `Rc`, `Arc`, `RefCell`)
//!
//! ## Key Backend Insights:
//! - **`Box<T>`**: Heap allocation. Essential for recursive types (e.g. JSON AST / filters)
//!   and dynamic dispatch (`Box<dyn Error>`, `Box<dyn Service>`).
//! - **The `Drop` Trait (RAII)**: Resource Acquisition Is Initialization. Used to automatically
//!   close database connections, commit/rollback transactions, and log request duration guards.
//! - **`Rc<T>` vs `Arc<T>`**:
//!   - `Rc<T>`: Single-threaded reference counting (cheap, but compiler forbids passing across threads).
//!   - `Arc<T>`: Atomic Reference Counting (thread-safe, universal standard for shared backend state).
//! - **`RefCell<T>` & Interior Mutability**: Mutating data behind an immutable reference at runtime.
//!   Useful for mock objects and caches in single-threaded tests.

use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

// --- 1. Box<T> & Recursive Data Types ---

/// Recursive tree structure representing a nested JSON or filter expression:
/// `AND(Age > 18, Status == 'Active')`
#[derive(Debug, PartialEq, Eq)]
pub enum QueryFilter {
    Leaf(String),
    And(Box<QueryFilter>, Box<QueryFilter>),
    Or(Box<QueryFilter>, Box<QueryFilter>),
}

// --- 2. The Drop Trait & RAII Request Timing Guard ---

/// An RAII guard that measures how long an HTTP request or database query took,
/// automatically printing/recording the duration when dropped at the end of the scope.
pub struct RequestTimerGuard {
    pub endpoint: String,
    start_time: Instant,
}

impl RequestTimerGuard {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            start_time: Instant::now(),
        }
    }
}

impl Drop for RequestTimerGuard {
    fn drop(&mut self) {
        let elapsed = self.start_time.elapsed();
        println!(
            "[TIMER AUDIT] Route '{}' completed in {:?}",
            self.endpoint, elapsed
        );
    }
}

// --- 3. Custom Smart Pointer with Deref Trait ---

/// Transparent wrapper around a value providing custom logging or audit metadata.
pub struct Audited<T> {
    value: T,
    pub accessed_by: String,
}

impl<T> Audited<T> {
    pub fn new(value: T, user: impl Into<String>) -> Self {
        Self {
            value,
            accessed_by: user.into(),
        }
    }
}

impl<T> Deref for Audited<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

// --- 4. RefCell<T> for Interior Mutability (Mocking in Tests) ---

pub trait NotificationService {
    fn send_sms(&self, phone: &str, message: &str);
}

/// Mock service using `RefCell` to record sent messages even though `&self` is immutable!
#[derive(Default)]
pub struct MockNotificationService {
    pub sent_messages: RefCell<Vec<(String, String)>>,
}

impl NotificationService for MockNotificationService {
    fn send_sms(&self, phone: &str, message: &str) {
        // `borrow_mut()` obtains mutable access at runtime:
        self.sent_messages
            .borrow_mut()
            .push((phone.to_string(), message.to_string()));
    }
}

// --- 5. Rc<T> vs Arc<T> ---

pub fn reference_counting_demo() {
    println!("\n=== Reference Counting: Rc vs Arc ===");

    // Single-threaded: Rc<T>
    let shared_config = Rc::new(String::from("db_pool_size=50"));
    let reader_1 = Rc::clone(&shared_config);
    let reader_2 = Rc::clone(&shared_config);
    println!(
        "Rc strong reference count: {}",
        Rc::strong_count(&shared_config)
    );
    drop(reader_1);
    println!(
        "Rc strong count after dropping reader_1: {}",
        Rc::strong_count(&shared_config)
    );
    drop(reader_2);

    // Multi-threaded safe: Arc<T>
    let thread_safe_config = Arc::new(String::from("max_threads=8"));
    let arc_clone = Arc::clone(&thread_safe_config);
    println!(
        "Arc strong reference count: {}",
        Arc::strong_count(&thread_safe_config)
    );
    drop(arc_clone);
}

fn main() {
    println!("=== Chapter 15: Smart Pointers in Backend Architecture ===");

    // 1. Recursive Box demo
    let filter = QueryFilter::And(
        Box::new(QueryFilter::Leaf("age > 21".to_string())),
        Box::new(QueryFilter::Or(
            Box::new(QueryFilter::Leaf("role == 'admin'".to_string())),
            Box::new(QueryFilter::Leaf("role == 'editor'".to_string())),
        )),
    );
    println!("Constructed recursive AST filter via Box:\n{filter:#?}");

    // 2. Deref coercion demo
    let audited_email = Audited::new(String::from("customer@enterprise.com"), "admin_user");
    // Deref coercion: we pass &audited_email to a function expecting &str!
    println!(
        "Audited email length: {} bytes (audited by {})",
        audited_email.len(),
        audited_email.accessed_by
    );

    // 3. Drop guard demo
    {
        let _guard = RequestTimerGuard::new("/api/v1/checkout");
        // Simulate minor work:
        std::thread::sleep(std::time::Duration::from_millis(10));
        println!("Handler finished execution. Exiting block to trigger Drop guard...");
    } // `_guard` is dropped here, printing the timing audit automatically!

    reference_counting_demo();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recursive_query_filter() {
        let left = Box::new(QueryFilter::Leaf("status == 'ACTIVE'".to_string()));
        let right = Box::new(QueryFilter::Leaf("balance > 0".to_string()));
        let compound = QueryFilter::And(left, right);

        match compound {
            QueryFilter::And(l, r) => {
                assert_eq!(*l, QueryFilter::Leaf("status == 'ACTIVE'".to_string()));
                assert_eq!(*r, QueryFilter::Leaf("balance > 0".to_string()));
            }
            _ => panic!("Expected And variant"),
        }
    }

    #[test]
    fn test_deref_smart_pointer() {
        let aud = Audited::new("payload_data".to_string(), "system");
        assert_eq!(aud.deref(), "payload_data");
        assert_eq!(aud.accessed_by, "system");
    }

    #[test]
    fn test_refcell_mock_service() {
        let mock = MockNotificationService::default();
        mock.send_sms("+1234567890", "Your OTP is 4242");

        let messages = mock.sent_messages.borrow();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].0, "+1234567890");
        assert_eq!(messages[0].1, "Your OTP is 4242");
    }
}
