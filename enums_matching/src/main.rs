//! # Chapter 6: Enums, Pattern Matching, `Option<T>` & `let-else`
//!
//! ## Key Backend Insights:
//! - **No `null` / `undefined`**: Rust eliminates NullPointerExceptions entirely using `Option<T>`.
//!   Optional database columns or query parameters are represented as `Option<String>`.
//! - **Sum Types with Payloads**: Enums can hold data, enabling type-safe state machines
//!   (e.g., Payment workflows, Order statuses).
//! - **Exhaustive Pattern Matching**: Adding a new enum variant forces you at compile time
//!   to handle it in every match expression across your entire codebase.
//! - **`let-else` Guard Clauses**: Idiomatic Rust (Edition 2021+) for clean early-returns
//!   in backend handlers without nested `if let` pyramids.

// --- 1. Enums with Data (Backend State Machine) ---

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaymentStatus {
    Pending,
    Authorized {
        auth_code: String,
    },
    Captured {
        transaction_id: String,
        amount_cents: u64,
    },
    Refunded {
        refund_id: String,
        amount_cents: u64,
        reason: String,
    },
    Failed {
        error_code: String,
        message: String,
    },
}

impl PaymentStatus {
    /// Determines whether the payment represents finalized funds.
    pub fn is_settled(&self) -> bool {
        matches!(self, PaymentStatus::Captured { .. })
    }

    /// Generates a user-facing summary using exhaustive pattern matching.
    pub fn summary(&self) -> String {
        match self {
            PaymentStatus::Pending => "Payment is pending customer authorization.".to_string(),
            PaymentStatus::Authorized { auth_code } => {
                format!("Payment authorized with code: {auth_code}")
            }
            PaymentStatus::Captured {
                transaction_id,
                amount_cents,
            } => {
                let dollars = *amount_cents as f64 / 100.0;
                format!("Successfully captured ${dollars:.2} (Tx: {transaction_id})")
            }
            PaymentStatus::Refunded {
                refund_id,
                amount_cents,
                reason,
            } => {
                let dollars = *amount_cents as f64 / 100.0;
                format!("Refunded ${dollars:.2} ({reason}, Refund ID: {refund_id})")
            }
            PaymentStatus::Failed {
                error_code,
                message,
            } => {
                format!("Payment failed [{error_code}]: {message}")
            }
        }
    }
}

// --- 2. Option<T> & Guard Clauses with `let-else` ---

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub path: String,
    pub authorization_header: Option<String>,
}

/// Simulated backend authentication handler demonstrating `let-else`.
/// If the token is missing or malformed, early-return an HTTP 401 response.
pub fn authenticate_request(req: &HttpRequest) -> Result<&str, (u16, &'static str)> {
    // 1. Guard against missing header using `let-else`
    let Some(auth_val) = &req.authorization_header else {
        return Err((401, "Missing Authorization header"));
    };

    // 2. Guard against missing Bearer prefix
    let Some(token) = auth_val.strip_prefix("Bearer ") else {
        return Err((
            401,
            "Invalid Authorization scheme: expected 'Bearer <token>'",
        ));
    };

    // 3. Simple token validation
    if token.is_empty() {
        return Err((401, "Bearer token cannot be empty"));
    }

    Ok(token)
}

// --- 3. Option Combinators: map, unwrap_or, and_then ---

/// Simulated repository lookup returning Option<u32>.
pub fn lookup_user_discount(user_id: u64) -> Option<u32> {
    if user_id == 1 {
        Some(15) // 15% discount for user 1
    } else {
        None // No discount for others
    }
}

pub fn option_combinators_demo() {
    println!("\n=== Option Combinators ===");

    // Transform value inside Option with .map():
    let formatted = lookup_user_discount(1).map(|d| format!("{d}% OFF"));
    println!("Formatted discount for user 1: {formatted:?}");

    // Provide default fallback with .unwrap_or():
    let fallback = lookup_user_discount(99).unwrap_or(0);
    println!("Default discount value for user 99: {fallback}%");
}

fn main() {
    println!("=== Chapter 6: Enums, Pattern Matching & Option ===");

    let pending = PaymentStatus::Pending;
    let captured = PaymentStatus::Captured {
        transaction_id: "tx_abc123".to_string(),
        amount_cents: 4999,
    };
    let failed = PaymentStatus::Failed {
        error_code: "INSUFFICIENT_FUNDS".to_string(),
        message: "Card was declined by issuing bank".to_string(),
    };

    println!("Pending:  {}", pending.summary());
    println!("Captured: {}", captured.summary());
    println!("Is captured settled? {}", captured.is_settled());
    println!("Failed:   {}", failed.summary());

    // Testing Request Authentication
    println!("\n=== Backend Auth Guards with let-else ===");
    let valid_req = HttpRequest {
        path: "/api/v1/profile".to_string(),
        authorization_header: Some("Bearer secure_jwt_token_999".to_string()),
    };
    let invalid_req = HttpRequest {
        path: "/api/v1/profile".to_string(),
        authorization_header: None,
    };

    println!("Valid Request: {:?}", authenticate_request(&valid_req));
    println!("Missing Auth:  {:?}", authenticate_request(&invalid_req));

    option_combinators_demo();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_status_captured() {
        let status = PaymentStatus::Captured {
            transaction_id: "tx_1".to_string(),
            amount_cents: 2500,
        };
        assert!(status.is_settled());
        assert_eq!(status.summary(), "Successfully captured $25.00 (Tx: tx_1)");
    }

    #[test]
    fn test_authenticate_request_success() {
        let req = HttpRequest {
            path: "/dashboard".to_string(),
            authorization_header: Some("Bearer my_secret_token".to_string()),
        };
        assert_eq!(authenticate_request(&req), Ok("my_secret_token"));
    }

    #[test]
    fn test_authenticate_request_missing_header() {
        let req = HttpRequest {
            path: "/dashboard".to_string(),
            authorization_header: None,
        };
        assert_eq!(
            authenticate_request(&req),
            Err((401, "Missing Authorization header"))
        );
    }

    #[test]
    fn test_authenticate_request_invalid_scheme() {
        let req = HttpRequest {
            path: "/dashboard".to_string(),
            authorization_header: Some("Basic user:pass".to_string()),
        };
        assert_eq!(
            authenticate_request(&req),
            Err((
                401,
                "Invalid Authorization scheme: expected 'Bearer <token>'"
            ))
        );
    }
}
