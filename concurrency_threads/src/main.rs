//! # Chapter 16: Fearless Concurrency (Threads, Channels & Shared State)
//!
//! ## Key Backend Insights:
//! - **Fearless Concurrency**: The Rust compiler guarantees at compile time that your code has
//!   ZERO data races. If two threads could simultaneously write to the same memory without a lock,
//!   Rust refuses to compile.
//! - **Message Passing (`std::sync::mpsc`)**: "Do not communicate by sharing memory; instead, share
//!   memory by communicating." Channels are the standard for background job queues and worker pools.
//! - **Shared State (`Arc<Mutex<T>>`)**: When multiple threads must access and mutate shared state
//!   (e.g., active connection counts, rate limiter windows, shared caches), wrap data in `Arc<Mutex<T>>`.
//! - **`Send` and `Sync`**: Marker traits. Almost all primitive and standard types are `Send` + `Sync`.
//!   Types with non-atomic internals (`Rc`, `RefCell`) are not `Send`/`Sync`, stopping thread bugs cold.

use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

// --- 1. Background Job Queue using MPSC Channels ---

#[derive(Debug, PartialEq, Eq)]
pub struct Job {
    pub id: u64,
    pub task_name: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct JobResult {
    pub job_id: u64,
    pub success: bool,
    pub worker_id: usize,
}

/// Dispatches jobs to multiple background threads, collecting results via an MPSC channel.
pub fn process_jobs_concurrently(jobs: Vec<Job>, num_workers: usize) -> Vec<JobResult> {
    let (tx, rx) = mpsc::channel();
    let jobs_arc = Arc::new(Mutex::new(jobs));
    let mut handles = vec![];

    for worker_id in 0..num_workers {
        let thread_tx = tx.clone();
        let thread_jobs = Arc::clone(&jobs_arc);

        let handle = thread::spawn(move || {
            loop {
                // Safely pop a job from the shared vector
                let maybe_job = {
                    let mut lock = thread_jobs.lock().unwrap();
                    lock.pop()
                };

                match maybe_job {
                    Some(job) => {
                        // Simulate work:
                        thread::sleep(Duration::from_millis(5));
                        let result = JobResult {
                            job_id: job.id,
                            success: true,
                            worker_id,
                        };
                        thread_tx.send(result).unwrap();
                    }
                    None => break, // No more jobs left
                }
            }
        });
        handles.push(handle);
    }

    // Drop original sender so the receiver knows when all threads are done
    drop(tx);

    for handle in handles {
        handle.join().unwrap();
    }

    // Collect all results from channel
    rx.into_iter().collect()
}

// --- 2. Shared-State Concurrency with Arc<Mutex<T>> ---

/// Thread-safe in-memory rate limiter tracking active connection counts.
#[derive(Clone, Default)]
pub struct ThreadSafeCounter {
    counter: Arc<Mutex<u64>>,
}

impl ThreadSafeCounter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment(&self) -> u64 {
        let mut count = self.counter.lock().unwrap();
        *count += 1;
        *count
    }

    pub fn get(&self) -> u64 {
        *self.counter.lock().unwrap()
    }
}

fn main() {
    println!("=== Chapter 16: Concurrency, Channels & Mutexes ===");

    // 1. Shared Mutex Counter across 10 threads
    println!("\n=== 1. Spawning 10 Concurrent Counter Threads ===");
    let counter = ThreadSafeCounter::new();
    let mut thread_handles = vec![];

    for i in 0..10 {
        let counter_clone = counter.clone();
        let handle = thread::spawn(move || {
            let val = counter_clone.increment();
            println!("  [Thread {i:2}] Incremented counter to: {val}");
        });
        thread_handles.push(handle);
    }

    for handle in thread_handles {
        handle.join().expect("Thread should not panic");
    }

    println!("Final Counter Value: {}", counter.get());

    // 2. MPSC Worker Pool Demo
    println!("\n=== 2. MPSC Background Worker Pool Demo ===");
    let jobs = vec![
        Job {
            id: 101,
            task_name: "Send Welcome Email".to_string(),
        },
        Job {
            id: 102,
            task_name: "Generate Monthly Invoice".to_string(),
        },
        Job {
            id: 103,
            task_name: "Sync Stripe Subscriptions".to_string(),
        },
        Job {
            id: 104,
            task_name: "Process Image Thumbnail".to_string(),
        },
    ];

    let results = process_jobs_concurrently(jobs, 3);
    println!(
        "Processed {} background jobs across 3 workers:",
        results.len()
    );
    for res in results {
        println!(
            "  Worker {} completed Job #{} (success: {})",
            res.worker_id, res.job_id, res.success
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_safe_counter() {
        let counter = ThreadSafeCounter::new();
        let mut handles = vec![];

        for _ in 0..20 {
            let c = counter.clone();
            handles.push(thread::spawn(move || {
                c.increment();
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(counter.get(), 20);
    }

    #[test]
    fn test_job_processing_pool() {
        let jobs = vec![
            Job {
                id: 1,
                task_name: "Job 1".to_string(),
            },
            Job {
                id: 2,
                task_name: "Job 2".to_string(),
            },
            Job {
                id: 3,
                task_name: "Job 3".to_string(),
            },
        ];

        let results = process_jobs_concurrently(jobs, 2);
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.success));
    }
}
