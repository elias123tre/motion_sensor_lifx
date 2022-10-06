use std::time::Duration;

/// Signals that can be sent to a [`Timer`]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SIGNAL<T> {
    /// Start the timer, will reset the countdown
    START,
    /// Terminate whole timer thread
    TERMINATE,
    /// Send arbitrary message to socket
    OTHER(T),
}

/// Actions that can be received in the callback of a [`Timer`]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ACTION {
    /// If restarted while already running
    START { restarted: bool },
    /// If a timeout is reached in [`Timer`]
    TIMEOUT,
}

/// Timeout for the PIR timer
pub const TIMEOUT: Duration = Duration::from_secs(60 * 10); // 10 minutes
/// Timeout for UDP socket read and write
pub const SOCKET_TIMEOUT: Duration = Duration::from_secs(30);

/// Duration the light takes to completely turn off after no motion for [`TIMEOUT`] time
pub const FADE_DURATION: Duration = Duration::from_secs(60 * 3); // 3 minutes
/// HSBK color for when light is off/dark after fading, by modifying input color
pub const fn fade_target(color: HSBK) -> HSBK {
    HSBK {
        brightness: light::MIN,
        ..color
    }
}
/// Float percentage factor that fading color should match within for it to appear as not-changed
pub const MATCHING_THRESHOLD: f32 = 0.05; // 5%

/// IP address of ceiling light
pub const TAKLAMPA: &str = "192.168.1.11:56700";
/// IP address of light strip
pub const LIFXZ: &str = "192.168.1.12:56700";

pub use lifx_core::Message;
use lifx_core::HSBK;

pub mod timer;
pub use timer::Timer;

pub mod light;
pub use light::Light;

pub mod temperature;

mod buffer;
pub use buffer::FixedBuffer;
