#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SIGNAL<T> {
    START,
    STOP,
    OTHER(T),
}

pub mod timer;
pub use timer::Timer;
