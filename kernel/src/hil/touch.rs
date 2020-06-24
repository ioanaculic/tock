//! Interface for touch input devices

use crate::ReturnCode;

#[derive(Debug)]
pub enum TouchEvent {
    Pressed,
    Released,
}

pub trait Touch {
    fn enable(&self) -> ReturnCode;
    fn disable(&self) -> ReturnCode;

    fn set_client(&self, touch_client: &'static dyn TouchClient);
}

pub trait MultiTouch {
    /// Subscribe to one of the touches
    fn subscribe_to_touch(id: usize) -> ReturnCode;

    /// Subscribe to all touches
    fn subscribe_to_all() -> ReturnCode;

    /// Retruns the number of concurently supported touches
    fn get_num_touches() -> ReturnCode;
}

pub trait TouchClient {
    fn touch_event(&self, event: TouchEvent, x: usize, y: usize);
}

pub trait MultiTouchClient {
    fn touch(id: usize, x: usize, y: usize);
    fn touch_move(id: usize, x: usize, y: usize);
    fn touch_up(id: usize, x: usize, y: usize);
}

pub trait Gesture {}
