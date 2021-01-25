//! Tock syscall driver capsule for Alarms, which issue callbacks when
//! a point in time has been reached.
use core::cell::Cell;
use core::mem;
use kernel::debug;
use kernel::hil::time::{Counter, Ticks, Ticks16, Freq16KHz, Frequency};
use kernel::{AppId, Callback, CommandResult, Driver, ErrorCode, Grant};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Timer as usize;

#[derive(Clone)]
pub struct TimerData {
    callback: Callback,
    last_value: Cell<u16> 
}

impl Default for TimerData {
    fn default() -> TimerData {
        TimerData {
            callback: Callback::default(),
            last_value: Cell::new(0)
        }
    }
}

pub struct TimerDriver<'a, C: Counter<'a>> {
    counter: &'a C,
    app: Grant<TimerData>
}

impl<'a, C: Counter<'a>> TimerDriver<'a, C> {
    pub const fn new(counter: &'a C, app: Grant<TimerData>) -> TimerDriver<'a, C> {
        TimerDriver {
             counter: counter,
             app: app
        }
    }
}

impl<'a, C: Counter<'a>> Driver for TimerDriver<'a, C> {
    /// Subscribe to timer expiration
    ///
    /// ### `_subscribe_num`
    ///
    /// - `0`: Subscribe to timer
    fn subscribe(
        &self,
        subscribe_num: usize,
        mut callback: Callback,
        app_id: AppId,
    ) -> Result<Callback, (Callback, ErrorCode)> {
        match subscribe_num {
            0 => {
                let res = self
                    .app
                    .enter(app_id, |td, _allocator| {
                        mem::swap(&mut td.callback, &mut callback);
                    })
                    .map_err(ErrorCode::from);
                
                if let Err(e) = res {
                    Err((callback, e))
                } else {
                    Ok(callback)
                }
            }
            _ => Err((callback, ErrorCode::NOSUPPORT)),
        }
    }

    fn command(&self, cmd_num: usize, _arg1: usize, _: usize, _appid: AppId) -> CommandResult{
        match cmd_num {
            0 => CommandResult::success(),
            1 => {
                let res = self
                    .app
                    .enter(_appid, |td, _allocator| {
                        let now = self.counter.now();
                        td.last_value.set(now.into_u32() as u16);
                        td.callback.schedule(0, now.into_usize(), 0);
                        // debug!("Command 1, now: {:?}", now.into_usize());
                    })
                    .map_err(ErrorCode::from);

                if let Err(e) = res {
                    CommandResult::failure(e)
                } else {
                    CommandResult::success()
                }
            },
            2 => {
                let mut diff_ms: usize = 0;
                // debug!("Command 2 inceput");
                let res = self
                    .app
                    .enter(_appid, |td, _allocator| {
                        let last_value: Ticks16 = Ticks16::from(td.last_value.get());
                        // debug!("Command 2 dupa last_value");
                        let new: Ticks16 = Ticks16::from(self.counter.now().into_u32());
                        let mut diff: Ticks16;

                        if new.into_u32() < last_value.into_u32() {
                            diff = Ticks16::max_value().wrapping_sub(last_value);
                            diff = diff.wrapping_add(new);
                        }
                        else { 
                            diff = new.wrapping_sub(last_value);
                        }

                        diff_ms = ((diff.into_u32() * 1000000)/ Freq16KHz::frequency()) as usize;

                        // debug!("Command 2, diff_ms: {:?}", diff_ms);
                        // debug!("Command 2, new: {:?}", new.into_usize());
                        td.callback.schedule(0, new.into_usize(), diff_ms);
                    })
                    .map_err(ErrorCode::from);

                if let Err(e) = res {
                    // debug!("Error: {:?}", e);
                    CommandResult::failure(e)
                } else {
                    // debug!("Succes cica? {:?}", diff_ms);
                    CommandResult::success()
                }
            },
            _ => CommandResult::failure(ErrorCode::NOSUPPORT)
        }
    }
}
