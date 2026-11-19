//! # Chapter 17 & Ecosystem: Asynchronous Programming with Tokio
//!
//! ## Key Backend Insights:
//! - **Why Async?**: In Chapter 16 and 21, each connection ran on an OS thread (~2–8 MB stack each).
//!   OS threads scale to hundreds or thousands before context switching degrades throughput.
//!   **Tokio green tasks** use only ~300 bytes of memory per task, allowing a single server to handle
//!   **hundreds of thousands of concurrent I/O connections**.
//! - **Futures are Lazy**: Calling `fetch_user(id)` does zero work until `.await` is reached!
//! - **Cooperative Multitasking**: Async tasks yield when awaiting I/O (`.await`). Never run heavy
//!   CPU-blocking calculations or `std::thread::sleep` on a Tokio worker thread without `tokio::task::spawn_blocking`.
//! - **Cancellation Safety**: When an HTTP client drops a connection, Tokio cancels the handler future.
//!   Code must handle cancellation gracefully without leaking resources.

use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{sleep, timeout};

// --- 1. Concurrent Aggregation with tokio::join! ---

#[derive(Debug, PartialEq, Eq)]
pub struct UserProfile {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct BillingStatus {
    pub is_paid: bool,
    pub plan: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DashboardData {
    pub user: UserProfile,
    pub billing: BillingStatus,
}

/// Simulated async database query.
pub async fn fetch_user_profile(user_id: u64) -> Result<UserProfile, &'static str> {
    // Non-blocking sleep: yields thread back to Tokio runtime
    sleep(Duration::from_millis(15)).await;

    if user_id == 0 {
        return Err("User not found");
    }

    Ok(UserProfile {
        id: user_id,
        name: format!("User_{user_id}"),
    })
}

/// Simulated external billing API call.
pub async fn fetch_billing_status(_user_id: u64) -> Result<BillingStatus, &'static str> {
    sleep(Duration::from_millis(25)).await;

    Ok(BillingStatus {
        is_paid: true,
        plan: "Enterprise".to_string(),
    })
}

/// Fetches user profile and billing concurrently using `tokio::join!`.
/// Instead of taking 15ms + 25ms = 40ms sequentially,
/// both run simultaneously and complete in max(15ms, 25ms) ≈ 25ms!
pub async fn get_user_dashboard(user_id: u64) -> Result<DashboardData, &'static str> {
    let (user_result, billing_result) =
        tokio::join!(fetch_user_profile(user_id), fetch_billing_status(user_id));

    let user = user_result?;
    let billing = billing_result?;

    Ok(DashboardData { user, billing })
}

// --- 2. Timeout & Cancellation Safety ---

pub async fn query_with_timeout(
    duration_ms: u64,
    timeout_limit_ms: u64,
) -> Result<&'static str, &'static str> {
    let slow_operation = async {
        sleep(Duration::from_millis(duration_ms)).await;
        "Operation Succeeded"
    };

    match timeout(Duration::from_millis(timeout_limit_ms), slow_operation).await {
        Ok(result) => Ok(result),
        Err(_) => Err("Request timed out (HTTP 504 Gateway Timeout)"),
    }
}

// --- 3. Async MPSC Channels for Event Streaming ---

pub async fn run_async_event_stream() -> Vec<String> {
    let (tx, mut rx) = mpsc::channel::<String>(32);

    // Producer task spawned on background Tokio worker
    tokio::spawn(async move {
        for event in [
            "UserRegistered",
            "EmailVerificationSent",
            "AccountActivated",
        ] {
            sleep(Duration::from_millis(5)).await;
            if tx.send(event.to_string()).await.is_err() {
                break;
            }
        }
    });

    let mut received = vec![];
    while let Some(event) = rx.recv().await {
        received.push(event);
    }
    received
}

#[tokio::main]
async fn main() {
    println!("=== Chapter 17: Asynchronous Programming with Tokio ===");

    // 1. Concurrent I/O demo (tokio::join!)
    println!("\n=== 1. Concurrent Dashboard Aggregation ===");
    let start = std::time::Instant::now();
    match get_user_dashboard(42).await {
        Ok(dashboard) => {
            println!("Fetched dashboard in {:?}:", start.elapsed());
            println!("  User:    {:?}", dashboard.user);
            println!("  Billing: {:?}", dashboard.billing);
        }
        Err(e) => eprintln!("Error fetching dashboard: {e}"),
    }

    // 2. High concurrency green tasks demo
    println!("\n=== 2. Spawning 100 Lightweight Tokio Tasks ===");
    let task_start = std::time::Instant::now();
    let mut handles = vec![];

    for i in 1..=100 {
        let handle = tokio::spawn(async move {
            sleep(Duration::from_millis(10)).await;
            i * 2
        });
        handles.push(handle);
    }

    let mut sum = 0;
    for handle in handles {
        sum += handle.await.expect("Task should not fail");
    }
    println!(
        "Computed sum of 100 concurrent tasks: {sum} in {:?}",
        task_start.elapsed()
    );

    // 3. Timeout handling demo
    println!("\n=== 3. Timeout Protection ===");
    let fast = query_with_timeout(10, 50).await;
    println!("Fast query (10ms with 50ms limit): {fast:?}");

    let slow = query_with_timeout(100, 30).await;
    println!("Slow query (100ms with 30ms limit): {slow:?}");

    // 4. Async MPSC Stream
    println!("\n=== 4. Async MPSC Event Stream ===");
    let events = run_async_event_stream().await;
    for evt in events {
        println!("  Received async event: {evt}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_user_dashboard_success() {
        let res = get_user_dashboard(1)
            .await
            .expect("Dashboard fetch should succeed");
        assert_eq!(res.user.id, 1);
        assert_eq!(res.billing.plan, "Enterprise");
    }

    #[tokio::test]
    async fn test_get_user_dashboard_invalid_user() {
        let res = get_user_dashboard(0).await;
        assert_eq!(res, Err("User not found"));
    }

    #[tokio::test]
    async fn test_timeout_success() {
        let res = query_with_timeout(5, 50).await;
        assert_eq!(res, Ok("Operation Succeeded"));
    }

    #[tokio::test]
    async fn test_timeout_triggered() {
        let res = query_with_timeout(50, 5).await;
        assert_eq!(res, Err("Request timed out (HTTP 504 Gateway Timeout)"));
    }

    #[tokio::test]
    async fn test_async_event_stream() {
        let events = run_async_event_stream().await;
        assert_eq!(events.len(), 3);
        assert_eq!(events[0], "UserRegistered");
        assert_eq!(events[2], "AccountActivated");
    }
}
