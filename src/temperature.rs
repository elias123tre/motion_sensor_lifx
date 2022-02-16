use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use crate::FixedBuffer;

pub const BUFFER_LEN: usize = 10;

/// Temperature in degrees celsius
pub type Temp = f32;

/// Thermal zone for temperature reading
#[derive(Clone, Debug, PartialEq)]
pub struct Thermal {
    pub temperature_file: PathBuf,
    pub interval: Duration,
    /// Fixed buffer of readings with size [`BUFFER_LEN`]
    readings: FixedBuffer<Option<Temp>, BUFFER_LEN>,
}

impl Thermal {
    pub fn new(temperature_file: PathBuf, interval: Duration) -> Self {
        Self {
            temperature_file,
            interval,
            readings: FixedBuffer::default(),
        }
    }

    /// Get current temperature
    pub fn get_temp(&self) -> Result<Temp, Box<dyn Error>> {
        let temp: i32 = fs::read_to_string(&self.temperature_file)?.trim().parse()?;
        Ok(temp as Temp / 1000.0)
    }

    /// Polling loop for temperature, executing callback every interval and stopping on message from receiver
    pub fn event_loop<T>(&mut self, callback: fn(Vec<Temp>) -> (), receiver: Receiver<T>) -> () {
        loop {
            self.readings.push(Some(self.get_temp().unwrap()));
            let values: Vec<Option<Temp>> = self.readings.into();
            let values: Vec<Temp> = values
                .iter()
                .filter_map(|c| c.and_then(|c| Some(c)))
                .collect();
            callback(values);
            match receiver.recv_timeout(self.interval) {
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => (),
                _ => break,
            }
        }
    }

    pub fn has_increased(readings: &Vec<Temp>) -> bool {
        // TODO: implement if increased x amount of times
        if readings.len() < 2 {
            false
        } else {
            readings[0] > readings[1]
        }
    }
}

impl Default for Thermal {
    fn default() -> Self {
        Self {
            temperature_file: PathBuf::from("/sys/class/thermal/thermal_zone0/temp"),
            interval: Duration::from_millis(200),
            readings: FixedBuffer::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::mpsc, thread, time::Instant};

    #[test]
    fn test_get_temp_x20() {
        let proc = Thermal::default();
        for _ in 0..20 {
            let before = Instant::now();
            let temp = proc.get_temp().unwrap();
            println!("{} {:?}", temp, before.elapsed());
        }
    }

    #[test]
    fn test_get_temp_loop() {
        let mut proc = Thermal::default();
        let (sender, receiver) = mpsc::channel::<()>();
        let print = |val| println!("{} {:?}", Thermal::has_increased(&val), val);
        let handle = thread::spawn(move || proc.event_loop(print, receiver));
        thread::sleep(Duration::from_secs(5));
        sender.send(()).unwrap();
        handle.join().unwrap();
    }
}
