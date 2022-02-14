use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub struct Processor {
    pub temperature_file: PathBuf,
}

impl Default for Processor {
    fn default() -> Self {
        Self {
            temperature_file: PathBuf::from("/sys/class/thermal/thermal_zone0/temp"),
        }
    }
}
