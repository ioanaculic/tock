//! Provides userspace with access to the touch panel.
//!
//! Usage
//! -----
//!
//! You need a screen that provides the `hil::touch::Touch` trait.
//!
//! ```rust
//! let touch =
//!     components::touch::TouchComponent::new(board_kernel, ts).finalize(());
//! ```

// use core::convert::From;
use kernel::debug;
use kernel::hil;
use kernel::hil::touch::{TouchEvent, TouchStatus, GestureEvent};
use kernel::ReturnCode;
use kernel::{AppId, Callback, Driver, Grant};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Touch as usize;

pub struct App {
    callback: Option<Callback>,
}

impl Default for App {
    fn default() -> App {
        App { callback: None }
    }
}

pub struct Touch<'a> {
    touch: &'a dyn hil::touch::Touch,
    apps: Grant<App>,
}

impl<'a> Touch<'a> {
    pub fn new(touch: &'a dyn hil::touch::Touch, grant: Grant<App>) -> Touch<'a> {
        Touch {
            touch: touch,
            apps: grant,
        }
    }
}

impl<'a> hil::touch::TouchClient for Touch<'a> {
    fn touch_event(&self, event: TouchEvent) {
        // debug!("touch {:?} x {} y {} area {:?} weight {:?}", event.status, event.x, event.y, event.area, event.weight);
        for app in self.apps.iter() {
            app.enter(|app, _| {
                app.callback.map(|mut callback| {
                    let event_id = match event.status {
                        TouchStatus::Released => 0,
                        TouchStatus::Pressed => 1,
                    };
                    callback.schedule(event.x, event.y, event_id);
                })
            });
        }
    }
}

impl<'a> hil::touch::GestureClient for Touch<'a> {
    fn gesture_event(&self, event: GestureEvent) {
        debug!("gesture {:?}", event);
        // for app in self.apps.iter() {
        //     app.enter(|app, _| {
        //         app.callback.map(|mut callback| {
        //             let event_id = match event.status {
        //                 TouchStatus::Released => 0,
        //                 TouchStatus::Pressed => 1,
        //             };
        //             callback.schedule(event.x, event.y, event_id);
        //         })
        //     });
        // }
    }
}

impl<'a> Driver for Touch<'a> {
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        app_id: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            0 => self
                .apps
                .enter(app_id, |app, _| {
                    app.callback = callback;
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn command(
        &self,
        command_num: usize,
        _data1: usize,
        _data2: usize,
        _appid: AppId,
    ) -> ReturnCode {
        match command_num {
            0 =>
            // This driver exists.
            {
                ReturnCode::SUCCESS
            }

            // Enable
            1 => self.touch.enable(),
            // Disable
            2 => self.touch.disable(),

            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
