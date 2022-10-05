use std::time::{Duration, Instant};

use gpio_cdev::{Chip, EventRequestFlags, EventType, LineRequestFlags};
use lifx_core::HSBK;

use motion_sensor_lifx::{
    fade_target, light::matches_fade, Light, Timer, ACTION, FADE_DURATION, TAKLAMPA, TIMEOUT,
};

fn main() -> Result<(), gpio_cdev::Error> {
    let mut chip = Chip::new("/dev/gpiochip0")?;
    let pin = 17;
    // Error will appear here if line is occupied
    let line = chip
        .get_line(pin)
        .expect(&format!("GPIO Line {} is occupied", pin));

    // Get iterator over input events from line
    let events = line
        .events(
            LineRequestFlags::INPUT,
            EventRequestFlags::BOTH_EDGES,
            "rust-program",
        )
        .expect(&format!("Unable to receive events on GPIO line {}", pin));

    let light = Light::new(TAKLAMPA)?;

    // Is Some of (before fade color, instant fading started) if currently fading
    let mut before_fade: Option<(HSBK, Instant)> = None;

    let timer = Timer::new(TIMEOUT, move |action| match action {
        ACTION::START { restarted: false } => {
            println!("Started!");
            // if fading
            if let Some((before_color, fading_started)) = before_fade {
                light
                    .change_color(
                        |current_color| {
                            if matches_fade(
                                before_color,
                                fade_target(before_color),
                                current_color,
                                fading_started.elapsed(),
                                FADE_DURATION,
                            ) {
                                println!("Light on from faded state");
                                before_color
                            } else {
                                println!("Light changed during fade or off");
                                current_color
                            }
                        },
                        Duration::from_millis(100),
                    )
                    .unwrap_or_else(|e| todo!("handle set color error gracefully: {:?}", e));
            }
            before_fade = None;
        }
        ACTION::START { restarted: true } => println!("Restarted!"),
        ACTION::TIMEOUT => {
            println!("Timeout!");
            light
                .change_color(
                    |color| {
                        // save color before fade, to be able to restore
                        before_fade = Some((color, Instant::now()));
                        fade_target(color)
                    },
                    FADE_DURATION,
                )
                .unwrap_or_else(|e| todo!("handle set color error gracefully: {:?}", e));
        }
    });

    println!("Program started and waiting for events on GPIO pin {}", pin);

    // Wait for GPIO events, this loop will go forever
    for event in events {
        let evt = event?;
        match evt.event_type() {
            // If PIR detects motion
            EventType::RisingEdge => {
                println!("Motion on");
                // Stop timer
                timer.start().unwrap();
            }
            // If PIR detects no motion for ~10 seconds
            EventType::FallingEdge => {
                println!("Motion off");
                // Restart timer
                timer.start().unwrap();
            }
        }
    }
    eprintln!("Program reached end, no events in gpio_cdev Iterator");
    Ok(())
}
