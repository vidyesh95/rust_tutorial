//! # Chapter 13: Functional Language Features (Iterators & Closures)
//!
//! ## Key Backend Insights:
//! - **Zero-Cost Abstractions**: Rust's iterators compile down to the same or faster assembly
//!   as hand-written indexed loops because the compiler eliminates bounds checking.
//! - **Lazy Evaluation**: Iterators do no work until consumed (e.g. via `.collect()`, `.sum()`, `.fold()`).
//!   This allows composing complex data transformations without intermediate allocations.
//! - **Closure Traits (`Fn`, `FnMut`, `FnOnce`)**:
//!   - `Fn`: Borrows values from the environment immutably (`&T`).
//!   - `FnMut`: Borrows values mutably (`&mut T`).
//!   - `FnOnce`: Takes ownership with `move`. Required when sending closures across threads (`tokio::spawn`).

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionStatus {
    Completed,
    Pending,
    Failed,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    pub id: u64,
    pub user_id: u64,
    pub amount_cents: u64,
    pub status: TransactionStatus,
    pub currency: String,
}

// --- 1. Closures & Capturing Modes ---

pub fn closures_demo() {
    println!("\n=== 1. Closures & Captures ===");

    let threshold = 5000; // in cents ($50.00)

    // Immutable capture (implements `Fn`):
    let is_high_value = |tx: &Transaction| tx.amount_cents >= threshold;

    let sample_tx = Transaction {
        id: 1,
        user_id: 10,
        amount_cents: 7500,
        status: TransactionStatus::Completed,
        currency: "USD".to_string(),
    };

    println!(
        "Transaction {} is high value: {}",
        sample_tx.id,
        is_high_value(&sample_tx)
    );

    // Mutable capture (implements `FnMut`):
    let mut total_processed = 0;
    let mut count_processor = || {
        total_processed += 1;
        total_processed
    };

    count_processor();
    count_processor();
    println!("Total processed via FnMut closure: {total_processed}");
}

// --- 2. Iterators & Functional Data Processing Pipeline ---

/// Filters completed transactions for a specific user and sums the total cents.
pub fn calculate_user_total(transactions: &[Transaction], target_user_id: u64) -> u64 {
    transactions
        .iter()
        .filter(|tx| tx.user_id == target_user_id && tx.status == TransactionStatus::Completed)
        .map(|tx| tx.amount_cents)
        .sum()
}

/// Demonstrates `filter_map`: parsing raw header lines into valid (key, value) pairs,
/// discarding invalid lines cleanly without errors.
pub fn parse_valid_headers(raw_lines: &[&str]) -> Vec<(String, String)> {
    raw_lines
        .iter()
        .filter_map(|line| {
            let mut parts = line.splitn(2, ':');
            let key = parts.next()?.trim();
            let value = parts.next()?.trim();
            if key.is_empty() || value.is_empty() {
                None
            } else {
                Some((key.to_string(), value.to_string()))
            }
        })
        .collect()
}

/// Backend Pagination using `.skip()` and `.take()`.
pub fn paginate_transactions(
    transactions: &[Transaction],
    page: usize,
    page_size: usize,
) -> Vec<&Transaction> {
    let skip_count = page.saturating_sub(1) * page_size;
    transactions
        .iter()
        .skip(skip_count)
        .take(page_size)
        .collect()
}

fn main() {
    println!("=== Chapter 13: Functional Iterators & Closures ===");
    closures_demo();

    let transactions = vec![
        Transaction {
            id: 1,
            user_id: 100,
            amount_cents: 2500,
            status: TransactionStatus::Completed,
            currency: "USD".to_string(),
        },
        Transaction {
            id: 2,
            user_id: 100,
            amount_cents: 1500,
            status: TransactionStatus::Pending,
            currency: "USD".to_string(),
        },
        Transaction {
            id: 3,
            user_id: 200,
            amount_cents: 10000,
            status: TransactionStatus::Completed,
            currency: "USD".to_string(),
        },
        Transaction {
            id: 4,
            user_id: 100,
            amount_cents: 4000,
            status: TransactionStatus::Completed,
            currency: "USD".to_string(),
        },
    ];

    let user_100_total = calculate_user_total(&transactions, 100);
    println!(
        "\nUser 100 Completed Total: ${:.2}",
        user_100_total as f64 / 100.0
    );

    // Header Parsing via filter_map
    let raw_headers = [
        "Content-Type: application/json",
        "X-Request-Id: req_xyz789",
        "Invalid-Header-No-Colon",
        "Empty-Value: ",
        "Authorization: Bearer token123",
    ];
    let parsed_headers = parse_valid_headers(&raw_headers);
    println!("\nParsed Headers via filter_map ({parsed_headers:?}):");
    for (k, v) in &parsed_headers {
        println!("  {k}: {v}");
    }

    // Pagination demo
    let page_1 = paginate_transactions(&transactions, 1, 2);
    println!(
        "\nPage 1 (2 items): {:?}",
        page_1.iter().map(|t| t.id).collect::<Vec<_>>()
    );
    let page_2 = paginate_transactions(&transactions, 2, 2);
    println!(
        "Page 2 (2 items): {:?}",
        page_2.iter().map(|t| t.id).collect::<Vec<_>>()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_user_total() {
        let txs = vec![
            Transaction {
                id: 1,
                user_id: 1,
                amount_cents: 1000,
                status: TransactionStatus::Completed,
                currency: "USD".to_string(),
            },
            Transaction {
                id: 2,
                user_id: 1,
                amount_cents: 2000,
                status: TransactionStatus::Failed,
                currency: "USD".to_string(),
            },
            Transaction {
                id: 3,
                user_id: 1,
                amount_cents: 3000,
                status: TransactionStatus::Completed,
                currency: "USD".to_string(),
            },
        ];
        assert_eq!(calculate_user_total(&txs, 1), 4000);
        assert_eq!(calculate_user_total(&txs, 999), 0);
    }

    #[test]
    fn test_parse_valid_headers() {
        let headers = ["Host: api.example.com", "BadLine", "Accept: text/html"];
        let parsed = parse_valid_headers(&headers);
        assert_eq!(parsed.len(), 2);
        assert_eq!(
            parsed[0],
            ("Host".to_string(), "api.example.com".to_string())
        );
        assert_eq!(parsed[1], ("Accept".to_string(), "text/html".to_string()));
    }

    #[test]
    fn test_pagination() {
        let txs: Vec<Transaction> = (1..=10)
            .map(|id| Transaction {
                id,
                user_id: 1,
                amount_cents: 100,
                status: TransactionStatus::Completed,
                currency: "USD".to_string(),
            })
            .collect();

        let page1 = paginate_transactions(&txs, 1, 3);
        assert_eq!(page1.len(), 3);
        assert_eq!(page1[0].id, 1);
        assert_eq!(page1[2].id, 3);

        let page4 = paginate_transactions(&txs, 4, 3);
        assert_eq!(page4.len(), 1); // 10th item
        assert_eq!(page4[0].id, 10);
    }
}
