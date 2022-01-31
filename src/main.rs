use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use gpio_cdev::{Chip, EventRequestFlags, EventType, LineRequestFlags};

#[derive(Clone, Copy, Debug, PartialEq)]
enum SIGNAL<T> {
    START,
    STOP,
    #[allow(dead_code)]
    OTHER(T),
}

const TIMEOUT: Duration = Duration::from_secs(30);

fn main() -> Result<(), gpio_cdev::Error> {
    let mut chip = Chip::new("/dev/gpiochip0")?;
    // Error will appear here if line is occupied
    let line = chip.get_line(17)?;

    // Get iterator over input events from line
    let events = line.events(
        LineRequestFlags::INPUT,
        EventRequestFlags::BOTH_EDGES,
        "rust-program",
    )?;

    // Create sender and receiver to communicate with timer thread
    let (send, recv) = mpsc::channel::<SIGNAL<String>>();

    // Create forever running timer thread that listens on channel
    thread::spawn(move || {
        let mut running = false;
        // Keep the thread alive, always check for next signal
        loop {
            // Wait for signal or timeout, whichever comes first
            match recv.recv_timeout(TIMEOUT) {
                Ok(SIGNAL::STOP) => {
                    println!("Thread stopped (timer cancelled)");
                    running = false;
                }
                Ok(SIGNAL::START) => {
                    println!("Thread restarted (timer started)");
                    running = true;
                }
                // Arbitrary message received
                Ok(SIGNAL::OTHER(message)) => {
                    println!("Signal sent to thread with message: {}", message)
                }
                // Signal receiving timed out
                Err(mpsc::RecvTimeoutError::Timeout) if running => {
                    println!("Thread has been running for (more than) 5 secs, blocking until next signal is start");
                    // Block until start signal is received
                    loop {
                        match recv.recv() {
                            Ok(SIGNAL::START) => {
                                println!("Thread restarted after timeout (timer started)");
                                running = true;
                                break;
                            }
                            Ok(SIGNAL::OTHER(message)) => {
                                println!("Signal sent to thread with message: {}", message)
                            }
                            Ok(SIGNAL::STOP) => {} // Nothing happens if already stopped
                            Err(_) => panic!("Channel has hung up"),
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {} // Ignore timeout while timer not running
                Err(_) => {
                    panic!("Channel has hung up")
                }
            }
        }
    });

    // Wait for GPIO events, this loop will go forever
    for event in events {
        let evt = event?;
        match evt.event_type() {
            // If PIR detects motion
            EventType::RisingEdge => {
                println!("Motion on");
                // Stop timer
                send.send(SIGNAL::STOP)
                    .expect("Thread stopped, cannot send signal");
            }
            // If PIR detects no motion for ~10 seconds
            EventType::FallingEdge => {
                println!("Motion off");

                // Stop timer
                send.send(SIGNAL::STOP)
                    .expect("Thread stopped, cannot send signal");
                // Then restart timer
                send.send(SIGNAL::START)
                    .expect("Thread stopped, cannot send signal");
            }
        }
    }
    Ok(())
}
