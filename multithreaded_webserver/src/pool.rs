//! A fixed-size thread pool backed by a channel of boxed jobs.

use std::sync::{Arc, Mutex, mpsc};
use std::thread;

/// A job that one worker runs exactly once.
type Job = Box<dyn FnOnce() + Send + 'static>;

/// The shared receiving end of the job channel.
type JobReceiver = Arc<Mutex<mpsc::Receiver<Job>>>;

/// A pool of worker threads that pull jobs from one channel.
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    /// Creates a pool with `size` worker threads.
    ///
    /// # Panics
    ///
    /// Panics when `size` is zero.
    #[must_use]
    pub fn new(size: usize) -> Self {
        assert!(size > 0, "a thread pool needs at least one worker");

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let workers = (0..size)
            .map(|id| Worker::new(id, Arc::clone(&receiver)))
            .collect();

        Self {
            workers,
            sender: Some(sender),
        }
    }

    /// Queues `job` for the next free worker.
    pub fn execute<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let sender = self
            .sender
            .as_ref()
            .expect("thread pool has been shut down");
        sender.send(Box::new(job)).expect("a worker is alive");
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Dropping the sender closes the channel. Every worker's `recv()` then returns `Err` and
        // its loop exits; joining waits for the job that worker is currently running.
        self.sender = None;
        for worker in &mut self.workers {
            if let Some(handle) = worker.handle.take() {
                handle.join().expect("a worker thread panicked");
            }
        }
    }
}

/// One worker thread and its handle.
struct Worker {
    handle: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: JobReceiver) -> Self {
        let handle = thread::Builder::new()
            .name(format!("worker-{id}"))
            .spawn(move || {
                loop {
                    let job = receiver
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .recv();

                    match job {
                        Ok(job) => job(),
                        Err(_) => break, // the pool was dropped
                    }
                }
            })
            .expect("failed to spawn a worker thread");

        Self {
            handle: Some(handle),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn runs_every_queued_job() {
        let pool = ThreadPool::new(2);
        let (sender, receiver) = mpsc::channel();

        for _ in 0..4 {
            let sender = sender.clone();
            pool.execute(move || {
                sender.send(()).expect("test receiver is alive");
            });
        }

        for _ in 0..4 {
            receiver
                .recv_timeout(Duration::from_secs(2))
                .expect("every job should run");
        }
    }

    #[test]
    #[should_panic(expected = "at least one worker")]
    fn rejects_a_zero_sized_pool() {
        let _pool = ThreadPool::new(0);
    }
}
