use std::time::Duration;

use gpio_cdev::{Chip, EventRequestFlags, EventType, LineRequestFlags};
use motion_sensor_lifx::{Timer, ACTION, TIMEOUT};

fn main() -> Result<(), gpio_cdev::Error> {
    let mut chip = Chip::new("/dev/gpiochip0")?;
    let pin = 17;
    // Error will appear here if line is occupied
    let line = chip.get_line(pin)?;

    // Get iterator over input events from line
    let events = line.events(
        LineRequestFlags::INPUT,
        EventRequestFlags::BOTH_EDGES,
        "rust-program",
    )?;

    let timer = Timer::new(TIMEOUT, |action| match action {
        ACTION::START {
            has_timed_out: true,
        } => println!("Start after timeout!"),
        ACTION::START { .. } => println!("Start!"),
        ACTION::STOP {
            already_stopped: true,
        } => (), // Do something if stopped after already timed out
        ACTION::STOP { .. } => println!("Stop!"),
        ACTION::TIMEOUT => println!("Timeout!"),
    });
    timer.set_timeout(Duration::from_secs(5)).unwrap();

    println!("Program started and waiting for events on GPIO pin {}", pin);

    // Wait for GPIO events, this loop will go forever
    for event in events {
        let evt = event?;
        match evt.event_type() {
            // If PIR detects motion
            EventType::RisingEdge => {
                println!("Motion on");
                // Stop timer
                timer.stop().expect("Thread stopped, cannot send signal");
            }
            // If PIR detects no motion for ~10 seconds
            EventType::FallingEdge => {
                println!("Motion off");
                // Restart timer
                timer.restart().expect("Thread stopped, cannot send signal");
            }
        }
    }
    Ok(())
}
