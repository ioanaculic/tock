/// Syscall driver number.
use crate::driver;
use core::cell::Cell;
use core::cmp;
use core::str;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::debug;
use kernel::hil::uart;
use kernel::{AppId, AppSlice, Callback, Driver, Grant, ReturnCode, Shared};
pub const DRIVER_NUM: usize = driver::NUM::EspSerial as usize;

#[derive(Default)]
pub struct App {
    write_callback: Option<Callback>,
    write_buffer: Option<AppSlice<Shared, u8>>,
    write_len: usize,
    write_remaining: usize, // How many bytes didn't fit in the buffer and still need to be printed.
    pending_write: bool,

    read_callback: Option<Callback>,
    read_buffer: Option<AppSlice<Shared, u8>>,
    read_len: usize,
}

pub static mut WRITE_BUF: [u8; 128] = [0; 128];
pub static mut READ_BUF: [u8; 128] = [0; 128];
pub static mut HELPER_BUF: [u8; 128] = [0; 128];

pub struct Link {
    _link_id: usize,
    app: OptionalCell<AppId>,
}

impl Link {
    pub fn new(id: usize) -> Link {
        Link {
            _link_id: id,
            app: OptionalCell::empty(),
        }
    }
}

pub struct EspSerial<'a> {
    uart: &'a dyn uart::UartData<'a>,
    apps: Grant<App>,
    tx_in_progress: OptionalCell<AppId>,
    tx_buffer: TakeCell<'static, [u8]>,
    rx_in_progress: OptionalCell<AppId>,
    rx_buffer: TakeCell<'static, [u8]>,
    rx_buffer_helper: TakeCell<'static, [u8]>,
    helper_pos: Cell<usize>,
    tx_in_progress_helper: Cell<bool>,
    new_line: Cell<bool>,
    links: [&'a Link; 5],
}

impl<'a> EspSerial<'a> {
    pub fn new(
        uart: &'a dyn uart::UartData<'a>,
        tx_buffer: &'static mut [u8],
        rx_buffer: &'static mut [u8],
        rx_buffer_helper: &'static mut [u8],
        links: [&'static Link; 5],
        grant: Grant<App>,
    ) -> EspSerial<'a> {
        let esp = EspSerial {
            uart: uart,
            apps: grant,
            tx_in_progress: OptionalCell::empty(),
            tx_buffer: TakeCell::new(tx_buffer),
            rx_in_progress: OptionalCell::empty(),
            rx_buffer: TakeCell::new(rx_buffer),
            rx_buffer_helper: TakeCell::new(rx_buffer_helper),
            helper_pos: Cell::new(0),
            tx_in_progress_helper: Cell::new(false),
            new_line: Cell::new(false),
            links: links,
        };
        esp.rx_buffer.take().map(|buffer| {
            let (_err, _opt) = esp.uart.receive_buffer(buffer, 1);
        });
        esp
    }

    fn get_linkid(&self, app_id: AppId) -> usize {
        let mut return_val = 404;
        for i in 0..self.links.len() {
            if self.links[i].app.is_none() {
                self.links[i].app.set(app_id);
                return_val = i;
                break;
            }
        }
        return_val as usize
    }

    fn send_bind(&self, app_id: AppId, app: &mut App, len: usize, link_id: usize) -> ReturnCode {
        self.links[link_id].app.set(app_id);
        self.send_new(app_id, app, len)
    }

    fn close_connection(&self, app_id: AppId, app: &mut App, len: usize, link_id: usize) -> ReturnCode {
        self.links[link_id].app.clear();
        self.send_new(app_id, app, len)
    }

    /// Internal helper function for setting up a new send transaction
    fn send_new(&self, app_id: AppId, app: &mut App, len: usize) -> ReturnCode {
        match app.write_buffer.take() {
            Some(slice) => {
                app.write_len = cmp::min(len, slice.len());
                app.write_remaining = app.write_len;
                self.send(app_id, app, slice);
                ReturnCode::SUCCESS
            }
            None => {
                ReturnCode::EBUSY
            },
        }
    }

    /// Internal helper function for continuing a previously set up transaction
    /// Returns true if this send is still active, or false if it has completed
    fn send_continue(&self, app_id: AppId, app: &mut App) -> Result<bool, ReturnCode> {
        if app.write_remaining > 0 {
            app.write_buffer
                .take()
                .map_or(Err(ReturnCode::ERESERVE), |slice| {
                    self.send(app_id, app, slice);
                    Ok(true)
                })
        } else {
            Ok(false)
        }
    }

    /// Internal helper function for sending data for an existing transaction.
    /// Cannot fail. If can't send now, it will schedule for sending later.
    fn send(&self, app_id: AppId, app: &mut App, slice: AppSlice<Shared, u8>) {
        if self.tx_in_progress.is_none() {
            self.tx_in_progress.set(app_id);
            self.rx_in_progress.set(app_id);
            self.tx_buffer.take().map(|buffer| {
                let mut transaction_len = app.write_remaining;
                for (i, c) in slice.as_ref()[0..cmp::min(app.write_remaining, slice.len())]
                    .iter()
                    .enumerate()
                {
                    if buffer.len() <= i {
                        break;
                    }
                    buffer[i] = *c;
                }
                // Check if everything we wanted to print
                // fit in the buffer.
                if app.write_remaining > buffer.len() {
                    transaction_len = buffer.len();
                    app.write_remaining -= buffer.len();
                } else {
                    app.write_remaining = 0;
                }
                let (_err, _opt) = self.uart.transmit_buffer(buffer, transaction_len);
            });
        } else {
            app.pending_write = true;
        }
        app.write_buffer = Some(slice);
    }

    /// Internal helper function for starting a receive operation
    fn receive_new(&self, app_id: AppId, app: &mut App, len: usize) -> ReturnCode {
        if self.rx_buffer.is_none() {
            // For now, we tolerate only one concurrent receive operation on this console.
            // Competing apps will have to retry until success.
            return ReturnCode::EBUSY;
        }

        match app.read_buffer {
            Some(ref slice) => {
                let read_len = cmp::min(len, slice.len());
                if read_len > self.rx_buffer.map_or(0, |buf| buf.len()) {
                    // For simplicity, impose a small maximum receive length
                    // instead of doing incremental reads
                    ReturnCode::EINVAL
                } else {
                    // Note: We have ensured above that rx_buffer is present
                    app.read_len = read_len;
                    self.rx_buffer.take().map(|buffer| {
                        self.rx_in_progress.set(app_id);
                        let (_err, _opt) = self.uart.receive_buffer(buffer, app.read_len);
                    });
                    ReturnCode::SUCCESS
                }
            }
            None => {
                // Must supply read buffer before performing receive operation
                ReturnCode::EINVAL
            }
        }
    }

    // use to see what you write on the console

    // fn write(&self, byte: u8) -> ReturnCode {
    //     if self.tx_in_progress_helper.get() {
    //         ReturnCode::EBUSY
    //     } else {
    //         // self.tx_in_progress_helper.set(true);
    //         // self.tx_buffer.take().map(|buffer| {
    //         //     buffer[0] = byte;
    //         //     self.uart.transmit_buffer(buffer, 1);
    //         // });
    //         ReturnCode::SUCCESS
    //     }
    // }

    // fn write_bytes(&self, bytes: &[u8]) -> ReturnCode {
    //     if self.tx_in_progress_helper.get() {
    //         ReturnCode::EBUSY
    //     } else {
    //         // self.tx_in_progress_helper.set(true);
    //         // self.tx_buffer.take().map(|buffer| {
    //         //     let len = cmp::min(bytes.len(), buffer.len());
    //             // Copy elements of `bytes` into `buffer`
    //             // (&mut buffer[..len]).copy_from_slice(&bytes[..len]);
    //             // self.uart.transmit_buffer(buffer, len);
    //         // });
    //         ReturnCode::SUCCESS
    //     }
    // }

    fn read_command(&self) {
        self.rx_buffer_helper.map(|buffer| {
            let mut terminator = 0;
            let len = buffer.len();
            for i in 0..len {
                if buffer[i] == 0 {
                    terminator = i;
                    break;
                }
            }
            if terminator > 0 {
                let cmd_str = str::from_utf8(&buffer[0..terminator]);
                match cmd_str {
                    Ok(s) => {
                        let clean_str = s.trim();
                        if clean_str.starts_with("OK") {
                            self.rx_in_progress.take().map(|appid| {
                                self.apps.enter(appid, |app, _| {
                                    app.read_callback.map(|mut cb| {
                                        cb.schedule(From::from(ReturnCode::SUCCESS), 0, 0);
                                    })
                                })
                            });
                        } else if clean_str.starts_with("ERROR") {
                            self.rx_in_progress.take().map(|appid| {
                                self.apps.enter(appid, |app, _| {
                                    app.read_callback.map(|mut cb| {
                                        cb.schedule(From::from(ReturnCode::FAIL), 0, 0);
                                    })
                                })
                            });
                        } else if clean_str.starts_with("ALREADY CONNECTED") {
                            self.rx_in_progress.take().map(|appid| {
                                self.apps.enter(appid, |app, _| {
                                    app.read_callback.map(|mut cb| {
                                        cb.schedule(From::from(ReturnCode::EALREADY), 0, 0);
                                    })
                                })
                            });
                        } else if clean_str.starts_with("+IPD"){
                            let mut link_id_pos = clean_str.get(5..6);
                            link_id_pos.take().map(|link_id| {
                                let mut message_start = clean_str.find(":");
                                message_start.take().map(|pos| {
                                    let mut message = clean_str.get((pos+1)..);
                                    message.take().map(|buffer| {
                                        let id = link_id.as_bytes();
                                        self.links[(id[0] - 48) as usize].app.map(|appid| {
                                            self.apps.enter(*appid, |app, _| {
                                                app.read_callback.map(|mut cb| {
                                                    let rx_buffer = buffer.as_bytes();
                                                    if let Some(mut app_buffer) = app.read_buffer.take() {
                                                        for (a, b) in app_buffer.iter_mut().zip(rx_buffer) {
                                                            *a = *b;
                                                        }
                                                        app.read_buffer.replace(app_buffer);
                                                        cb.schedule(From::from(ReturnCode::SUCCESS), buffer.len(), 0);
                                                    }
                                                })
                                            })
                                        })
                                    });
                                });                                    
                            });
                        } 
                        else if clean_str.starts_with("+CIFSR:STAIP") {
                            let mut ip = clean_str.get(14..(clean_str.len()-1));
                            ip.take().map(|buffer| {
                                self.rx_in_progress.take().map(|appid| {
                                    self.apps.enter(appid, |app, _| {
                                        app.read_callback.map(|mut cb| {
                                            let rx_buffer = buffer.as_bytes();
                                            if let Some(mut app_buffer) = app.read_buffer.take() {
                                                for (a, b) in app_buffer.iter_mut().zip(rx_buffer) {
                                                    *a = *b;
                                                }
                                                app.read_buffer.replace(app_buffer);
                                                cb.schedule(From::from(ReturnCode::SUCCESS), buffer.len(), 0);
                                            }
                                        })
                                    })
                                });
                            });
                        }
                    }
                    Err(_e) => debug!("Invalid command: {:?}", buffer),
                }
            }
        });
    }
}

impl Driver for EspSerial<'_> {
    /// Setup shared buffers.
    ///
    /// ### `allow_num`
    ///
    /// - `1`: Writeable buffer for write buffer
    /// - `2`: Writeable buffer for read buffer
    fn allow(
        &self,
        appid: AppId,
        allow_num: usize,
        slice: Option<AppSlice<Shared, u8>>,
    ) -> ReturnCode {
        match allow_num {
            1 => self
                .apps
                .enter(appid, |app, _| {
                    app.write_buffer = slice;
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),
            2 => self
                .apps
                .enter(appid, |app, _| {
                    app.read_buffer = slice;
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    /// Setup callbacks.
    ///
    /// ### `subscribe_num`
    ///
    /// - `1`: Write buffer completed callback
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        app_id: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            1 /* putstr/write_done */ => {
                self.apps.enter(app_id, |app, _| {
                    app.write_callback = callback;
                    ReturnCode::SUCCESS
                }).unwrap_or_else(|err| err.into())
            },
            2 /* getnstr done */ => {
                self.apps.enter(app_id, |app, _| {
                    app.read_callback = callback;
                    ReturnCode::SUCCESS
                }).unwrap_or_else(|err| err.into())
            },
            _ => ReturnCode::ENOSUPPORT
        }
    }

    /// Initiate serial transfers
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver check.
    /// - `1`: Transmits a buffer passed via `allow`, up to the length
    ///        passed in `arg1`
    /// - `2`: Receives into a buffer passed via `allow`, up to the length
    ///        passed in `arg1`
    /// - `3`: Cancel any in progress receives and return (via callback)
    ///        what has been received so far.
    fn command(&self, cmd_num: usize, arg1: usize, arg2: usize, appid: AppId) -> ReturnCode {
        match cmd_num {
            0 /* check if present */ => ReturnCode::SUCCESS,
            1 => {
                let len = arg1;
                let link_id = arg2;
                self.apps.enter(appid, |app, _| {
                    app.write_len = len;
                    self.send_bind(appid, app, len, link_id)
                }).unwrap_or_else(|err| err.into())
            },
            2 /* putstr */ => {
                let len = arg1;
                self.apps.enter(appid, |app, _| {
                    app.write_len = len;
                    self.send_new(appid, app, len)
                }).unwrap_or_else(|err| err.into())
            },
            3 /* getnstr */ => {
                let len = arg1;
                self.apps.enter(appid, |app, _| {
                    self.receive_new(appid, app, len)
                }).unwrap_or_else(|err| err.into())
            },
            4 /* abort rx */ => {
                self.uart.receive_abort();
                ReturnCode::SUCCESS
            },
            5 => {
                let len = arg1;
                let link_id = arg2;
                self.apps.enter(appid, |app, _| {
                    self.close_connection(appid, app, len, link_id)
                }).unwrap_or_else(|err| err.into())
            },
            6 => {
                let mut val = 0;
                self.apps.enter(appid, |_app, _| {
                    val = self.get_linkid(appid);
                    ReturnCode::SUCCESS
                }).unwrap_or_else(|err| err.into());
                if val == 404 {
                    ReturnCode::EBUSY
                } else {
                    ReturnCode::SuccessWithValue {
                        value: val as usize 
                    }  
                }
            }
            _ => ReturnCode::ENOSUPPORT
        }
    }
}

impl uart::TransmitClient for EspSerial<'_> {
    fn transmitted_buffer(&self, buffer: &'static mut [u8], _tx_len: usize, _rcode: ReturnCode) {
        // Either print more from the AppSlice or send a callback to the
        // application.
        self.tx_in_progress_helper.set(false);
        self.tx_buffer.replace(buffer);
        self.tx_in_progress.take().map(|appid| {
            self.apps.enter(appid, |app, _| {
                match self.send_continue(appid, app) {
                    Ok(more_to_send) => {
                        if !more_to_send {
                            // Go ahead and signal the application
                            let written = app.write_len;
                            app.write_len = 0;
                            app.write_callback.map(|mut cb| {
                                // debug!("callback write");
                                cb.schedule(From::from(ReturnCode::SUCCESS), written, 0);
                            });
                        }
                    }
                    Err(return_code) => {
                        // XXX This shouldn't ever happen?
                        app.write_len = 0;
                        app.write_remaining = 0;
                        app.pending_write = false;
                        let r0 = isize::from(return_code) as usize;
                        app.write_callback.map(|mut cb| {
                            cb.schedule(r0, 0, 0);
                        });
                    }
                }
            })
        });

        // If we are not printing more from the current AppSlice,
        // see if any other applications have pending messages.
        if self.tx_in_progress.is_none() {
            for cntr in self.apps.iter() {
                let started_tx = cntr.enter(|app, _| {
                    if app.pending_write {
                        app.pending_write = false;
                        match self.send_continue(app.appid(), app) {
                            Ok(more_to_send) => more_to_send,
                            Err(return_code) => {
                                // XXX This shouldn't ever happen?
                                app.write_len = 0;
                                app.write_remaining = 0;
                                app.pending_write = false;
                                let r0 = isize::from(return_code) as usize;
                                app.write_callback.map(|mut cb| {
                                    cb.schedule(r0, 0, 0);
                                });
                                false
                            }
                        }
                    } else {
                        false
                    }
                });
                if started_tx {
                    break;
                }
            }
        }
    }
}

impl uart::ReceiveClient for EspSerial<'_> {
    fn received_buffer(
        &self,
        buffer: &'static mut [u8],
        rx_len: usize,
        _rcode: ReturnCode,
        error: uart::Error,
    ) {
        if error == uart::Error::None {
            match rx_len {
                0 => debug!("ProcessConsole had read of 0 bytes"),
                1 => {
                    // uncomment write function calls to see what you write on the console
                    self.rx_buffer_helper.map(|command| {
                        let index = self.helper_pos.get() as usize;
                        if buffer[0] == ('\n' as u8) || buffer[0] == ('\r' as u8) {
                            // self.write_bytes(&['\r' as u8, '\n' as u8]);
                            self.new_line.set(true);
                            // self.write_bytes(command);
                        } else if buffer[0] == ('\x08' as u8) && index > 0 {
                            // Backspace, echo and remove last byte
                            // Note echo is '\b \b' to erase
                            // self.write_bytes(&['\x08' as u8, ' ' as u8, '\x08' as u8]);
                            command[index - 1] = '\0' as u8;
                            self.helper_pos.set(index - 1);
                        } else if index < (command.len() - 1) && buffer[0] < 128 {
                            // For some reason, sometimes reads return > 127 but no error,
                            // which causes utf-8 decoding failure, so check byte is < 128. -pal

                            // Echo the byte and store it
                            // self.write(buffer[0]);
                            command[index] = buffer[0];
                            self.helper_pos.set(index + 1);
                            command[index + 1] = 0;
                        }
                    });
                }
                _ => debug!(
                    "ProcessConsole issues reads of 1 byte, but receive_complete was length {}",
                    rx_len
                ),
            };
        }
        if self.new_line.get() == true {
            self.read_command();
            self.rx_buffer_helper.map(|buffer| {
                buffer[0] = 0;
            });
            self.helper_pos.set(0);
            self.new_line.set(false);
        }

        self.uart.receive_buffer(buffer, 1);
    }
}
