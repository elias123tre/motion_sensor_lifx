use std::time::Duration;

/// The signals that can be sent to a [`Timer`]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SIGNAL<T> {
    /// Start the timer, will reset the countdown
    START,
    /// Terminate whole timer thread
    TERMINATE,
    /// Send arbitrary message to socket
    OTHER(T),
}

/// The actions that can be received in the callback of a [`Timer`]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ACTION {
    /// If restarted while already running
    START { restarted: bool },
    /// If a timeout is reached in [`Timer`]
    TIMEOUT,
}

/// The timeout for the PIR timer
pub const TIMEOUT: Duration = Duration::from_secs(30);
/// The timeout for UDP socket read and write
pub const SOCKET_TIMEOUT: Duration = Duration::from_secs(30);

pub const TAKLAMPA: &str = "192.168.1.11:56700";
pub const LIFXZ: &str = "192.168.1.12:56700";

pub mod timer;
pub use timer::Timer;

pub mod light;
pub use light::Light;

pub mod temperature;

pub use lifx_core::Message;
