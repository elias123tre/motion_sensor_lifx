use std::sync::mpsc::Sender;
use std::sync::{mpsc, MutexGuard, PoisonError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use crate::{ACTION, SIGNAL};

pub type SignalResult = Result<(), mpsc::SendError<SIGNAL<String>>>;

/// A restartable timer
#[derive(Debug)]
pub struct Timer {
    thread: JoinHandle<()>,
    pub sender: Sender<SIGNAL<String>>,
    timeout: Arc<Mutex<Duration>>,
    running: Arc<Mutex<bool>>,
}

impl Timer {
    /// Create new timer with `timeout` and `callback`
    pub fn new<F: 'static + FnMut(ACTION) -> () + std::marker::Send>(
        timeout: Duration,
        mut callback: F,
    ) -> Self {
        let timeout_mutex = Arc::new(Mutex::new(timeout));
        let timeout_inner = timeout_mutex.clone();

        let running_mutex = Arc::new(Mutex::new(true));
        let running = running_mutex.clone();

        // Create sender and receiver to communicate with timer thread
        let (sender, receiver) = mpsc::channel();

        // Create forever running timer thread that listens on channel
        let thread = thread::Builder::new()
            .name("timer".to_string())
            .spawn(move || {
                // Keep the thread alive, always check for next signal
                'outer: loop {
                    // Wait for signal or timeout, whichever comes first
                    match receiver.recv_timeout(*timeout_inner.lock().unwrap()) {
                        Ok(SIGNAL::START) => {
                            callback(ACTION::START { restarted: true });
                            *running.lock().unwrap() = true;
                        }
                        Ok(SIGNAL::TERMINATE) => break 'outer,
                        // Arbitrary message received
                        Ok(SIGNAL::OTHER(message)) => {
                            println!("Signal received on thread with message: {}", message)
                        }
                        // Signal receiving timed out
                        Err(mpsc::RecvTimeoutError::Timeout) => {
                            let mut is_running = running.lock().unwrap();
                            if *is_running {
                                {
                                    callback(ACTION::TIMEOUT);
                                    *is_running = false;

                                    // release lock before blocking
                                    drop(is_running);

                                    // Block until start signal is received
                                    loop {
                                        match receiver.recv() {
                                            Ok(SIGNAL::START) => {
                                                callback(ACTION::START { restarted: false });
                                                *running.lock().unwrap() = true;
                                                break;
                                            }
                                            Ok(SIGNAL::TERMINATE) => break 'outer,
                                            Ok(SIGNAL::OTHER(message)) => {
                                                println!(
                                                    "Signal sent to thread with message: {}",
                                                    message
                                                )
                                            }
                                            Err(err) => panic!("Channel has hung up: {}", err),
                                        }
                                    }
                                }
                            }
                            // Ignore timeout while timer not running
                        }
                        Err(err) => panic!("Channel has hung up: {}", err),
                    }
                }
            })
            .unwrap();

        Self {
            thread,
            sender,
            running: running_mutex,
            timeout: timeout_mutex,
        }
    }

    /// Start the timer, restarting if already running
    pub fn start(&self) -> SignalResult {
        self.sender.send(SIGNAL::START)
    }
    /// Send a custom signal to the timer thread
    pub fn signal(&self, signal: SIGNAL<String>) -> SignalResult {
        self.sender.send(signal)
    }

    /// If the timer is counting down (running)
    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }

    /// Set the timer's timeout duration
    pub fn set_timeout(&self, timeout: Duration) -> Result<(), PoisonError<MutexGuard<Duration>>> {
        *self.timeout.lock()? = timeout;
        Ok(())
    }

    /// Get the current dereferenced timeout duration
    pub fn timeout(&self) -> Result<Duration, PoisonError<MutexGuard<'_, Duration>>> {
        Ok(*self.timeout.lock()?)
    }

    pub fn destroy(self) -> thread::Result<()> {
        self.sender.send(SIGNAL::TERMINATE).unwrap();
        self.thread.join()
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
    fn test_running() {
        let timer = Timer::new(Duration::from_millis(100), |_action| {});
        timer.start().unwrap();
        assert!(timer.is_running());
        thread::sleep(Duration::from_millis(500));
        assert!(!timer.is_running());
    }
    #[test]
    fn test_start() {
        let called = Arc::new(Mutex::new(false));
        let called_outer = called.clone();
        let timer = Timer::new(Duration::from_millis(100), move |action| {
            *called.lock().unwrap() = true;
            assert!(
                matches!(action, ACTION::START { .. }),
                "first action should be start"
            );
        });
        timer.start().unwrap();
        timer.destroy().unwrap();
        assert!(*called_outer.lock().unwrap(), "callback has been triggered");
    }
    #[test]
    fn test_timeout() {
        let actions: Arc<Mutex<[Option<ACTION>; 2]>> = Arc::new(Mutex::new([None; 2]));
        let actions_outer = actions.clone();
        let timer = Timer::new(Duration::from_millis(100), move |action| {
            let mut actions_inner = actions.lock().unwrap();
            if matches!(actions_inner[0], None) {
                (*actions_inner)[0] = Some(action);
            } else {
                (*actions_inner)[1] = Some(action);
            }
        });
        timer.start().unwrap();
        thread::sleep(Duration::from_millis(200));
        timer.destroy().unwrap();
        let actions_values = *actions_outer.lock().unwrap();
        assert_eq!(
            actions_values,
            [
                Some(ACTION::START { restarted: true }),
                Some(ACTION::TIMEOUT)
            ]
        );
    }
}
