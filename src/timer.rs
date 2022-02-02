use std::sync::mpsc::Sender;
use std::sync::{mpsc, MutexGuard, PoisonError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use crate::{ACTION, SIGNAL};

pub type SignalResult = Result<(), mpsc::SendError<SIGNAL<String>>>;

/// A cancellable/interruptable timer
#[derive(Debug)]
pub struct Timer {
    #[allow(dead_code)]
    thread: JoinHandle<()>,
    pub sender: Sender<SIGNAL<String>>,
    timeout: Arc<Mutex<Duration>>,
}

impl Timer {
    /// Create new timer with `timeout` and `callback`
    pub fn new<F: 'static + FnMut(ACTION) -> () + std::marker::Send>(
        timeout: Duration,
        mut callback: F,
    ) -> Self {
        let timeout_mutex = Arc::new(Mutex::new(timeout));
        let timeout_inner = Arc::clone(&timeout_mutex);

        // Create sender and receiver to communicate with timer thread
        let (sender, receiver) = mpsc::channel();

        // Create forever running timer thread that listens on channel
        let thread = thread::spawn(move || {
            let mut running = false;
            // Keep the thread alive, always check for next signal
            loop {
                // Wait for signal or timeout, whichever comes first
                match receiver.recv_timeout(*timeout_inner.lock().unwrap()) {
                    Ok(SIGNAL::STOP) => {
                        callback(ACTION::STOP {
                            already_stopped: false,
                        });
                        running = false;
                    }
                    Ok(SIGNAL::START) => {
                        callback(ACTION::START {
                            has_timed_out: false,
                        });
                        running = true;
                    }
                    // Arbitrary message received
                    Ok(SIGNAL::OTHER(message)) => {
                        println!("Signal received on thread with message: {}", message)
                    }
                    // Signal receiving timed out
                    Err(mpsc::RecvTimeoutError::Timeout) if running => {
                        callback(ACTION::TIMEOUT);

                        // Block until start signal is received
                        loop {
                            match receiver.recv() {
                                Ok(SIGNAL::START) => {
                                    callback(ACTION::START {
                                        has_timed_out: true,
                                    });
                                    running = true;
                                    break;
                                }
                                Ok(SIGNAL::STOP) => {
                                    // Nothing happens if already stopped but still activate callback
                                    callback(ACTION::STOP {
                                        already_stopped: true,
                                    });
                                }
                                Ok(SIGNAL::OTHER(message)) => {
                                    println!("Signal sent to thread with message: {}", message)
                                }
                                Err(err) => panic!("Channel has hung up: {}", err),
                            }
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {} // Ignore timeout while timer not running
                    Err(err) => panic!("Channel has hung up: {}", err),
                }
            }
        });

        Self {
            thread,
            sender,
            timeout: timeout_mutex,
        }
    }

    /// Start the timer
    pub fn start(&self) -> SignalResult {
        self.sender.send(SIGNAL::START)
    }
    /// Stop the timer
    pub fn stop(&self) -> SignalResult {
        self.sender.send(SIGNAL::STOP)
    }
    /// Stop the timer, then start it
    pub fn restart(&self) -> SignalResult {
        self.stop()?;
        self.start()?;
        Ok(())
    }
    /// Send a custom signal to the timer thread
    pub fn signal(&self, signal: SIGNAL<String>) -> SignalResult {
        self.sender.send(signal)
    }

    /// Set the timer's timeout
    pub fn set_timeout(&self, timeout: Duration) -> Result<(), PoisonError<MutexGuard<Duration>>> {
        *self.timeout.lock()? = timeout;
        Ok(())
    }

    /// Get a dereferenced timeout
    pub fn timeout(&self) -> Result<Duration, PoisonError<MutexGuard<'_, Duration>>> {
        Ok(*self.timeout.lock()?)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_creation() {
        let _timer = Timer::new(Duration::from_secs(5), |_action| {});
    }
    #[test]
    fn test_set_timeout() {
        let timer = Timer::new(Duration::from_secs(5), |_action| {});
        timer.set_timeout(Duration::from_secs(10)).unwrap();
        assert_eq!(timer.timeout().unwrap(), Duration::from_secs(10));
    }
    #[test]
    fn test_start() {
        let timer = Timer::new(Duration::from_secs(5), |action| {
            assert_eq!(
                action,
                ACTION::START {
                    has_timed_out: false
                }
            );
        });
        timer.start().unwrap();
    }
    #[test]
    fn test_stop() {
        let timer = Timer::new(Duration::from_secs(5), |action| {
            assert_eq!(
                action,
                ACTION::STOP {
                    already_stopped: true
                }
            );
        });
        timer.stop().unwrap();
    }
    #[test]
    fn test_timeout() {
        let timer = Timer::new(Duration::from_secs(5), |action| {
            assert_eq!(
                action,
                ACTION::START {
                    has_timed_out: false
                }
            );
        });
        timer.start().unwrap();
    }
}
