use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use crate::SIGNAL;

pub type SignalResult = Result<(), mpsc::SendError<SIGNAL<String>>>;
/// First argument is if timeout was reached before starting again
pub type OnStart = fn(bool) -> ();
/// First argument is if thread already is stopped
pub type OnStop = fn(bool) -> ();
pub type OnTimeout = fn() -> ();

/// A cancellable/interruptable timer
#[derive(Debug)]
pub struct Timer {
    thread: JoinHandle<()>,
    pub sender: Sender<SIGNAL<String>>,
    pub timeout: Arc<Mutex<Duration>>,
}

impl Timer {
    /// Create new timer
    ///
    /// * `on_start` - Function with first argument: has_timed_out
    /// * `on_stop` - Function with first argument: already_stopped
    /// * `on_timeout` - Function
    pub fn new(
        timeout: Duration,
        on_start: Option<OnStart>,
        on_stop: Option<OnStop>,
        on_timeout: Option<OnTimeout>,
    ) -> Self {
        let timeout_mutex = Arc::new(Mutex::new(timeout));
        let timeout_inner = Arc::clone(&timeout_mutex);

        // Create sender and receiver to communicate with timer thread
        let (sender, receiver) = mpsc::channel::<SIGNAL<String>>();

        // Create forever running timer thread that listens on channel
        let thread = thread::spawn(move || {
            let mut running = false;
            // Keep the thread alive, always check for next signal
            loop {
                // Wait for signal or timeout, whichever comes first
                match receiver.recv_timeout(*timeout_inner.lock().unwrap()) {
                    Ok(SIGNAL::STOP) => {
                        if let Some(stop) = on_stop {
                            stop(false);
                        }
                        running = false;
                    }
                    Ok(SIGNAL::START) => {
                        if let Some(start) = on_start {
                            start(false);
                        }
                        running = true;
                    }
                    // Arbitrary message received
                    Ok(SIGNAL::OTHER(message)) => {
                        println!("Signal received on thread with message: {}", message)
                    }
                    // Signal receiving timed out
                    Err(mpsc::RecvTimeoutError::Timeout) if running => {
                        if let Some(timeout) = on_timeout {
                            timeout();
                        }

                        // Block until start signal is received
                        loop {
                            match receiver.recv() {
                                Ok(SIGNAL::START) => {
                                    if let Some(start) = on_start {
                                        start(true);
                                    }
                                    running = true;
                                    break;
                                }
                                Ok(SIGNAL::STOP) => {
                                    if let Some(stop) = on_stop {
                                        stop(true);
                                    }
                                } // Nothing happens if already stopped
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
}
