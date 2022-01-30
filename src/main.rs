use std::time::Duration;
use std::{sync::mpsc, thread};

use gpio_cdev::{Chip, EventRequestFlags, EventType, LineRequestFlags};

#[allow(dead_code)]
enum SIGNAL<T> {
    START,
    STOP,
    OTHER(T),
}

fn main() -> Result<(), gpio_cdev::Error> {
    let mut chip = Chip::new("/dev/gpiochip0")?;
    let line = chip.get_line(17)?;
    println!("{:#?}", line.info());

    let events = line.events(
        LineRequestFlags::INPUT,
        EventRequestFlags::BOTH_EDGES,
        "rust-program",
    )?;

    let (send, recv) = mpsc::channel::<SIGNAL<String>>();
    thread::spawn(move || {
        let mut running = false;
        loop {
            match recv.recv_timeout(Duration::from_secs(5)) {
                Ok(SIGNAL::STOP) => {
                    println!("Thread stopped (timer cancelled)");
                    running = false;
                }
                Ok(SIGNAL::START) => {
                    println!("Thread restarted (timer started)");
                    running = true;
                }
                Ok(SIGNAL::OTHER(message)) => {
                    println!("Signal sent to thread with message: {}", message)
                }
                Err(mpsc::RecvTimeoutError::Timeout) if running => {
                    println!("Thread has been running for (more than) 5 secs, blocking until next signal is start");
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
                Err(mpsc::RecvTimeoutError::Timeout) => {} // Timed out while not running
                Err(_) => {
                    panic!("Channel has hung up")
                }
            }
        }
    });

    for event in events {
        let evt = event?;
        match evt.event_type() {
            EventType::RisingEdge => {
                println!("Motion on");
                // Stop timer
                send.send(SIGNAL::STOP)
                    .expect("Thread stopped, cannot send signal");
            }
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
