use std::time::Duration;

use gpio_cdev::{Chip, EventRequestFlags, EventType, LineRequestFlags};
use motion_sensor_lifx::Timer;

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

    let timer = Timer::new(
        TIMEOUT,
        Some(|has_timed_out| {
            if has_timed_out {
                println!("Start after timeout!");
            } else {
                println!("Start!");
            }
        }),
        Some(|already_stopped| {
            if !already_stopped {
                println!("Stop!");
            } else {
                // Do something if stopped after timeout
            }
        }),
        Some(|| {
            println!("Timeout!");
        }),
    );
    *timer.timeout.clone().lock().unwrap() = Duration::from_secs(5);

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
