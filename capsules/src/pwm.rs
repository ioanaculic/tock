use core::cell::Cell;
use core::cmp;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil;
use kernel::{AppId, AppSlice, Callback, Driver, Grant, ReturnCode, Shared};

use crate::driver;
use crate::virtual_pwm::Operation;
pub const DRIVER_NUM: usize = driver::NUM::Adc as usize;

pub struct PwmVirtualized<'a> {
    drivers: &'a [&'a dyn hil::pwm::PwmPin],
    apps: Grant<App>,
    current_app: OptionalCell<AppId>,
}

pub struct App {
    callback: Option<Callback>,
    pending_command: bool,
    command: OptionalCell<Operation>,
    channel: usize,
}

impl Default for App {
    fn default() -> AppSys {
        AppSys {
            callback: None,
            pending_command: false,
            command: OptionalCell::empty(),
            channel: 0,
        }
    }
}

impl<'a> PwmVirtualized<'a> {
    /// Create a new `Adc` application interface.
    ///
    /// - `drivers` - Virtual PWM drivers to provide application access to
    pub fn new(
        drivers: &'a [&'a dyn hil::pwm::PwmPin],
        grant: Grant<AppSys>,
    ) -> PwmVirtualized<'a> {
        PwmVirtualized {
            drivers: drivers,
            apps: grant,
            current_app: OptionalCell::empty(),
        }
    }

    /// Enqueue the command to be executed when the PWM is available.
    fn enqueue_command(&self, command: Operation, channel: usize, appid: AppId) -> ReturnCode {
        if channel < self.drivers.len() {
            self.apps
                .enter(appid, |app, _| {
                    if self.current_app.is_none() {
                        self.current_app.set(appid);
                        let value = self.call_driver(command, channel);
                        if value != ReturnCode::SUCCESS {
                            self.current_app.clear();
                        }
                        value
                    } else {
                        if app.pending_command == true {
                            ReturnCode::EBUSY
                        } else {
                            app.pending_command = true;
                            app.command.set(command);
                            app.channel = channel;
                            ReturnCode::SUCCESS
                        }
                    }
                })
                .unwrap_or_else(|err| err.into())
        } else {
            ReturnCode::ENODEVICE
        }
    }

    /// Request the sample from the specified channel
    fn call_driver(&self, command: Operation, channel: usize) -> ReturnCode {
        match command {
            Operation::Simple => {
                self.drivers[channel].start(command.frequency_hz, command.duty_cycle)
            }
            Operation::Stop => self.drivers[channel].stop(),
        }
    }
}
