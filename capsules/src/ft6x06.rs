//! Driver for the FT6x06 Touch Panel.
//!
//! I2C Interface
//!
//! <http://www.tvielectronics.com/ocart/download/controller/FT6206.pdf>
//!
//! Usage
//! -----
//!
//! ```rust
//! let mux_i2c = components::i2c::I2CMuxComponent::new(&stm32f4xx::i2c::I2C1)
//!     .finalize(components::i2c_mux_component_helper!());
//!
//! let ft6x06 = components::ft6x06::Ft6x06Component::new(
//!     stm32f412g::gpio::PinId::PG05.get_pin().as_ref().unwrap(),
//! )
//! .finalize(components::ft6x06_i2c_component_helper!(mux_i2c));
//!
//! Author: Alexandru Radovici <msg4alex@gmail.com>

#![allow(non_camel_case_types)]

use core::cell::Cell;
use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil::gpio;
use kernel::hil::bus::{self, Bus, BusWidth};
use kernel::hil::touch::{self, GestureEvent, TouchEvent, TouchStatus};
use kernel::{AppId, Driver, ReturnCode};

use crate::driver;

/// Syscall driver number.
pub const DRIVER_NUM: usize = driver::NUM::Ft6x06 as usize;

// Buffer to use for I2C messages
pub static mut BUFFER: [u8; 17] = [0; 17];

static NO_TOUCH: TouchEvent = TouchEvent {
    id: 0,
    x: 0,
    y: 0,
    status: TouchStatus::Released,
    size: None,
    pressure: None,
};

pub static mut EVENTS_BUFFER: [TouchEvent; 2] = [NO_TOUCH, NO_TOUCH];

#[derive(Copy, Clone)]
enum State {
    Idle,
    ReadingTouchesCmd,
    ReadingTouchesData,
}

enum_from_primitive! {
    enum Registers {
        REG_GEST_ID = 0x01,
        REG_TD_STATUS = 0x02,
        REG_CHIPID = 0xA3,
    }
}

pub struct Ft6x06<'a, B: Bus<'a>> {
    bus: &'a B,
    interrupt_pin: &'a dyn gpio::InterruptPin<'a>,
    touch_client: OptionalCell<&'a dyn touch::TouchClient>,
    gesture_client: OptionalCell<&'a dyn touch::GestureClient>,
    multi_touch_client: OptionalCell<&'a dyn touch::MultiTouchClient>,
    state: Cell<State>,
    num_touches: Cell<usize>,
    buffer: TakeCell<'static, [u8]>,
    events: TakeCell<'static, [TouchEvent]>,
}

impl<'a,B: Bus<'a>> Ft6x06<'a, B> {
    pub fn new(
        bus: &'a B,
        interrupt_pin: &'a dyn gpio::InterruptPin<'a>,
        buffer: &'static mut [u8],
        events: &'static mut [TouchEvent],
    ) -> Ft6x06<'a, B> {
        // setup and return struct
        interrupt_pin.enable_interrupts(gpio::InterruptEdge::FallingEdge);
        Ft6x06 {
            bus: bus,
            interrupt_pin: interrupt_pin,
            touch_client: OptionalCell::empty(),
            gesture_client: OptionalCell::empty(),
            multi_touch_client: OptionalCell::empty(),
            state: Cell::new(State::Idle),
            num_touches: Cell::new(0),
            buffer: TakeCell::new(buffer),
            events: TakeCell::new(events),
        }
    }
}

impl<'a,B: Bus<'a>> bus::Client for Ft6x06<'a, B> {
    fn command_complete(&self, buffer: Option<&'static mut [u8]>, _len: usize) {
        match self.state.get () {
            State::ReadingTouchesCmd => {
                self.state.set (State::ReadingTouchesData);
                self.buffer.take().map (|buffer| {
                    self.bus.read (BusWidth::Bits8, buffer, 15);
                });
            }
            State::ReadingTouchesData => {
                self.state.set(State::Idle);
                if let Some(buffer) = buffer {
        self.num_touches.set((buffer[0] & 0x0F) as usize);
        self.touch_client.map(|client| {
            if self.num_touches.get() <= 2 {
                let status = match buffer[0] >> 6 {
                    0x00 => TouchStatus::Pressed,
                    0x01 => TouchStatus::Released,
                    _ => TouchStatus::Released,
                };
                let x = (((buffer[1] & 0x0F) as u16) << 8) + (buffer[2] as u16);
                let y = (((buffer[3] & 0x0F) as u16) << 8) + (buffer[4] as u16);
                let pressure = Some(buffer[5] as u16);
                let size = Some(buffer[6] as u16);
                client.touch_event(TouchEvent {
                    status,
                    x,
                    y,
                    id: 0,
                    pressure,
                    size,
                });
            }
        });
        self.gesture_client.map(|client| {
            if self.num_touches.get() <= 2 {
                let gesture_event = match buffer[0] {
                    0x10 => Some(GestureEvent::SwipeUp),
                    0x14 => Some(GestureEvent::SwipeRight),
                    0x18 => Some(GestureEvent::SwipeDown),
                    0x1C => Some(GestureEvent::SwipeLeft),
                    0x48 => Some(GestureEvent::ZoomIn),
                    0x49 => Some(GestureEvent::ZoomOut),
                    _ => None,
                };
                if let Some(gesture) = gesture_event {
                    client.gesture_event(gesture);
                }
            }
        });
        self.multi_touch_client.map(|client| {
            if self.num_touches.get() <= 2 {
                for touch_event in 0..self.num_touches.get() {
                    let status = match buffer[0] >> 6 {
                        0x00 => TouchStatus::Pressed,
                        0x01 => TouchStatus::Released,
                        _ => TouchStatus::Released,
                    };
                    let x = (((buffer[1] & 0x0F) as u16) << 8) + (buffer[2] as u16);
                    let y = (((buffer[3] & 0x0F) as u16) << 8) + (buffer[4] as u16);
                    let pressure = Some(buffer[5] as u16);
                    let size = Some(buffer[6] as u16);
                    self.events.map(|buffer| {
                        buffer[touch_event] = TouchEvent {
                            status,
                            x,
                            y,
                            id: 0,
                            pressure,
                            size,
                        };
                    });
                }
                self.events.map(|buffer| {
                    client.touch_events(buffer, self.num_touches.get());
                });
            }
        });
        self.buffer.replace(buffer);
        self.interrupt_pin
            .enable_interrupts(gpio::InterruptEdge::FallingEdge);
        }
            }
            _ => panic! ("ft6x06 is idle")
        }
        
    }
}

impl<'a, B: Bus<'a>> gpio::Client for Ft6x06<'a, B> {
    fn fired(&self) {
        
            self.interrupt_pin.disable_interrupts();

            self.state.set(State::ReadingTouchesCmd);

        
            self.bus.set_addr(BusWidth::Bits8, Registers::REG_GEST_ID as usize);
    }
}

impl<'a, B: Bus<'a>> touch::Touch<'a> for Ft6x06<'a, B> {
    fn enable(&self) -> ReturnCode {
        ReturnCode::SUCCESS
    }

    fn disable(&self) -> ReturnCode {
        ReturnCode::SUCCESS
    }

    fn set_client(&self, client: &'a dyn touch::TouchClient) {
        self.touch_client.replace(client);
    }
}

impl<'a, B: Bus<'a>> touch::Gesture<'a> for Ft6x06<'a, B> {
    fn set_client(&self, client: &'a dyn touch::GestureClient) {
        self.gesture_client.replace(client);
    }
}

impl<'a, B: Bus<'a>> touch::MultiTouch<'a> for Ft6x06<'a, B> {
    fn enable(&self) -> ReturnCode {
        ReturnCode::SUCCESS
    }

    fn disable(&self) -> ReturnCode {
        ReturnCode::SUCCESS
    }

    fn get_num_touches(&self) -> usize {
        2
    }

    fn get_touch(&self, index: usize) -> Option<TouchEvent> {
        self.buffer.map_or(None, |buffer| {
            if index <= self.num_touches.get() {
                // a touch has 7 bytes
                let offset = index * 7;
                let status = match buffer[offset + 1] >> 6 {
                    0x00 => TouchStatus::Pressed,
                    0x01 => TouchStatus::Released,
                    _ => TouchStatus::Released,
                };
                let x = (((buffer[offset + 2] & 0x0F) as u16) << 8) + (buffer[offset + 3] as u16);
                let y = (((buffer[offset + 4] & 0x0F) as u16) << 8) + (buffer[offset + 5] as u16);
                let pressure = Some(buffer[offset + 6] as u16);
                let size = Some(buffer[offset + 7] as u16);
                Some(TouchEvent {
                    status,
                    x,
                    y,
                    id: 0,
                    pressure,
                    size,
                })
            } else {
                None
            }
        })
    }

    fn set_client(&self, client: &'a dyn touch::MultiTouchClient) {
        self.multi_touch_client.replace(client);
    }
}

impl<'a, B: Bus<'a>> Driver for Ft6x06<'a, B> {
    fn command(&self, command_num: usize, _: usize, _: usize, _: AppId) -> ReturnCode {
        match command_num {
            // is driver present
            0 => ReturnCode::SUCCESS,

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
