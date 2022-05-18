use std::error::Error;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};
use std::{fs, thread};

use crate::FixedBuffer;

pub const BUFFER_LEN: usize = 20;
pub const SCAN_INTERVAL: Duration = Duration::from_millis(100);
pub const SECONDS_HISTORY: u64 = BUFFER_LEN as u64 * SCAN_INTERVAL.as_secs();
// totals to a 2-second history

/// Temperature in degrees celsius
pub type Temp = f32;

/// Thermal zone for temperature reading
#[derive(Clone, Debug, PartialEq)]
pub struct Thermal {
    /// File to the sysfs thermal zone interface
    pub temperature_file: PathBuf,
    /// Interval between temperature readings
    pub interval: Duration,
    /// The time the temperature was last checked
    last_checked: Instant,
    /// Fixed buffer of readings with size [`BUFFER_LEN`]
    readings: FixedBuffer<Option<Temp>, BUFFER_LEN>,
}

impl Thermal {
    pub fn new(temperature_file: PathBuf, interval: Duration) -> Self {
        Self {
            temperature_file,
            interval,
            last_checked: Instant::now() - interval,
            readings: FixedBuffer::default(),
        }
    }

    /// Get current temperature
    pub fn get_temp(&self) -> Result<Temp, Box<dyn Error>> {
        let temp: i32 = fs::read_to_string(&self.temperature_file)?.trim().parse()?;
        Ok(temp as Temp / 1000.0)
    }

    /// Blocking polling loop for temperature, executing callback every interval with current temperature and stopping on any message from receiver
    pub fn event_loop<F, T>(&mut self, mut callback: F, receiver: Receiver<T>) -> ()
    where
        F: FnMut(Vec<Temp>) -> (),
    {
        loop {
            self.readings.push(Some(self.get_temp().unwrap()));
            let values: Vec<_> = self.readings.into_iter().collect();
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

    /// Get average of last `n` readings
    ///
    /// # Panics
    /// If no temperatures has been read yet
    pub fn average(&self, n: usize) -> Temp {
        let mut buffer = self.readings.into_iter().filter_map(|x| x);
        let first = buffer.next().expect("no temperature readings");
        let mut taken: usize = 1;
        buffer.take(n).fold(first, |acc, val| {
            taken += 1;
            acc + val
        }) / (taken as Temp)
    }

    pub fn moving_average() {
        // let fixed_buffer = self.readings.into_iter().filter_map(|s| s);
        // fixed_buffer
        //     .clone()
        //     .take(history)
        //     .zip(fixed_buffer.skip(1).take(history))
        //     .all(|(x, y)| x <= y);
    }
}

impl Default for Thermal {
    /// Default for raspberry pi
    fn default() -> Self {
        Self {
            temperature_file: PathBuf::from("/sys/class/thermal/thermal_zone0/temp"),
            interval: SCAN_INTERVAL,
            // initialize so temperature can be gotten immediately
            last_checked: Instant::now() - SCAN_INTERVAL,
            readings: FixedBuffer::default(),
        }
    }
}

impl Iterator for Thermal {
    type Item = Temp;

    /// Get current temperature, blocking until time since last reading is more than or equal to `self.interval`
    fn next(&mut self) -> Option<Self::Item> {
        let duration_since = Instant::now().duration_since(self.last_checked);
        if duration_since < self.interval {
            thread::sleep(self.interval - duration_since);
        }
        self.last_checked = Instant::now();
        let temp = self.get_temp().ok()?;
        self.readings.push(Some(temp));
        Some(temp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::thread;

    #[test]
    fn test_get_temp_x20() {
        let mut proc = Thermal::default();
        let before = Instant::now();
        let mut counter = 0;
        for temp in (&mut proc).take(20) {
            let elapsed = before.elapsed();
            println!("{} {:?}", temp, elapsed);
            assert!(elapsed >= SCAN_INTERVAL * counter);
            counter += 1;
        }
    }

    #[test]
    fn test_average() {
        let mut proc = Thermal::default();
        let temp = proc.next().unwrap();
        assert_eq!(proc.average(5), temp, "average after one reading");
        println!("{}", proc.average(3));
        for _ in 0..20 {
            proc.next().unwrap();
        }
        println!("{:?}", proc.readings.into_iter().collect::<Vec<_>>());
        println!("{}", proc.average(10));
    }

    #[test]
    fn test_moving_average() {
        let mut proc = Thermal::default();
        let (sender, receiver) = mpsc::channel::<()>();
        let print = move |val| {
            println!("{:?} {:?}", Thermal::moving_average(), val);
        };
        let handle = thread::spawn(move || proc.event_loop(print, receiver));
        thread::sleep(Duration::from_secs(10));
        sender.send(()).unwrap();
        handle.join().unwrap();
    }
}
