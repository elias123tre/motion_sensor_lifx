use std::time::Duration;

/// The signals that can be sent to a [`Timer`]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SIGNAL<T> {
    /// Start the timer
    START,
    /// Stop the timer
    STOP,
    /// Send arbitrary message to socket
    OTHER(T),
}

/// The actions that can be received in the callback of a [`Timer`]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ACTION {
    /// `has_timed_out` is if timeout was reached before starting again
    START { has_timed_out: bool },
    /// `already_stopped` is if timer is not even running when stop signal is sent
    STOP { already_stopped: bool },
    /// If a timeout is reached in [`Timer`]
    TIMEOUT,
}

/// The timeout for the PIR timer
pub const TIMEOUT: Duration = Duration::from_secs(30);
/// The timeout for UDP socket read and write
pub const SOCKET_TIMEOUT: Duration = Duration::from_secs(30);

pub mod timer;
pub use timer::Timer;

pub mod light;
pub use light::Light;

pub use lifx_core::Message;
