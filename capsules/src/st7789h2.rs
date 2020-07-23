//! ST7789H2 SPI Screen
//!
//! Usage
//! -----
//!
//! ```rust
//! let tft = components::st7789h2::ST7735Component::new(alarm_mux).finalize(
//!     components::st7735_component_helper!(
//!         // spi type
//!         stm32f4xx::spi::Spi,
//!         // chip select
//!         stm32f4xx::gpio::PinId::PE03,
//!         // spi mux
//!         spi_mux,
//!         // timer type
//!         stm32f4xx::tim2::Tim2,
//!         // dc pin
//!         stm32f4xx::gpio::PinId::PA00.get_pin().as_ref().unwrap(),
//!         // reset pin
//!         stm32f4xx::gpio::PinId::PA00.get_pin().as_ref().unwrap()
//!     )
//! );
//! ```

use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::debug;
use kernel::hil::gpio;
use kernel::hil::memory_async::{self, BusWidth};
use kernel::hil::screen::{
    self, ScreenClient, ScreenPixelFormat, ScreenRotation, ScreenSetupClient,
};
use kernel::hil::time::{self, Alarm, Frequency};
use kernel::ReturnCode;
use kernel::{AppId, Callback, Driver};

const BUFFER_SIZE: usize = 24;
pub static mut BUFFER: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

#[derive(Debug, PartialEq)]
pub struct Command {
    id: u8,
    parameters: Option<&'static [u8]>,
    delay: u8,
}

static NOP: Command = Command {
    id: 0x00,
    parameters: None,
    delay: 0,
};

static SW_RESET: Command = Command {
    id: 0x01,
    parameters: None,
    delay: 200,
};

static SLEEP_IN: Command = Command {
    id: 0x10,
    parameters: None,
    delay: 10,
};

static SLEEP_OUT: Command = Command {
    id: 0x11,
    parameters: None,
    delay: 120,
};

static DISPLAY_INVERSION: Command = Command {
    id: 0x21,
    parameters: None,
    delay: 120,
};

static DISPLAY_OFF: Command = Command {
    id: 0x28,
    parameters: None,
    delay: 0,
};

static DISPLAY_ON: Command = Command {
    id: 0x29,
    parameters: None,
    delay: 20,
};

static WRITE_RAM: Command = Command {
    id: 0x2C,
    parameters: None,
    delay: 0,
};

static READ_RAM: Command = Command {
    id: 0x2E,
    parameters: None,
    delay: 0,
};

static CASET: Command = Command {
    id: 0x2A,
    parameters: Some(&[0x00, 0x00, 0x00, 0xEF]),
    delay: 0,
};

static RASET: Command = Command {
    id: 0x2B,
    parameters: Some(&[0x00, 0x00, 0x00, 0xEF]),
    delay: 0,
};

static NORMAL_DISPLAY: Command = Command {
    id: 0x36,
    parameters: Some(&[0x00]),
    delay: 0,
};

static IDLE_OFF: Command = Command {
    id: 0x38,
    parameters: None,
    delay: 20,
};

static IDLE_ON: Command = Command {
    id: 0x39,
    parameters: None,
    delay: 0,
};

static COLOR_MODE: Command = Command {
    id: 0x3A,
    parameters: Some(&[0x05]),
    delay: 0,
};

static PV_GAMMA_CTRL: Command = Command {
    id: 0xE0,
    parameters: Some(&[
        0xD0, 0x08, 0x11, 0x08, 0x0C, 0x15, 0x39, 0x33, 0x50, 0x36, 0x13, 0x14, 0x29, 0x2D,
    ]),
    delay: 0,
};

static NV_GAMMA_CTRL: Command = Command {
    id: 0xE1,
    parameters: Some(&[
        0xD0, 0x08, 0x10, 0x08, 0x06, 0x06, 0x39, 0x44, 0x51, 0x0B, 0x16, 0x14, 0x2F, 0x31,
    ]),
    delay: 0,
};

static PORCH_CTRL: Command = Command {
    id: 0xB2,
    parameters: Some(&[0x0C, 0x0C, 0x00, 0x33, 0x33]),
    delay: 0,
};

static GATE_CTRL: Command = Command {
    id: 0xB7,
    parameters: Some(&[0x35]),
    delay: 0,
};

static LCM_CTRL: Command = Command {
    id: 0xC0,
    parameters: Some(&[0x2C]),
    delay: 0,
};

static VDV_VRH_EN: Command = Command {
    id: 0xC2,
    parameters: Some(&[0x01, 0xC3]),
    delay: 0,
};

static VDV_SET: Command = Command {
    id: 0xC4,
    parameters: Some(&[0x20]),
    delay: 0,
};

static FR_CTRL: Command = Command {
    id: 0xC6,
    parameters: Some(&[0x0F]),
    delay: 0,
};

static VCOM_SET: Command = Command {
    id: 0xBB,
    parameters: Some(&[0x1F]),
    delay: 0,
};

static POWER_CTRL: Command = Command {
    id: 0xD0,
    parameters: Some(&[0xA4, 0xA1]),
    delay: 0,
};

static TEARING_EFFECT: Command = Command {
    id: 0x35,
    parameters: Some(&[0x00]),
    delay: 0,
};

pub type CommandSequence = &'static [SendCommand];

macro_rules! default_parameters_sequence {
    ($($cmd:expr),+) => {
        [$(SendCommand::Default($cmd), )+]
    }
}

static INIT_SEQUENCE: [SendCommand; 21] = default_parameters_sequence!(
    &SLEEP_IN,
    &SW_RESET,
    &SLEEP_OUT,
    &NORMAL_DISPLAY,
    &COLOR_MODE,
    &DISPLAY_INVERSION,
    &CASET,
    &RASET,
    &PORCH_CTRL,
    &GATE_CTRL,
    &VCOM_SET,
    &LCM_CTRL,
    &VDV_VRH_EN,
    &VDV_SET,
    &FR_CTRL,
    &POWER_CTRL,
    &PV_GAMMA_CTRL,
    &NV_GAMMA_CTRL,
    &DISPLAY_ON,
    &SLEEP_OUT,
    &TEARING_EFFECT
);

static WRITE_PIXEL: [SendCommand; 3] = [
    SendCommand::Position(&CASET, 1, 4),
    SendCommand::Position(&RASET, 5, 4),
    SendCommand::Position(&WRITE_RAM, 9, 2),
];

const SEQUENCE_BUFFER_SIZE: usize = 24;
pub static mut SEQUENCE_BUFFER: [SendCommand; SEQUENCE_BUFFER_SIZE] =
    [SendCommand::Nop; SEQUENCE_BUFFER_SIZE];

#[derive(Copy, Clone, PartialEq)]
enum Status {
    Idle,
    Init,
    Reset1,
    Reset2,
    Reset3,
    Reset4,
    SendCommand(usize, usize, usize),
    SendCommandSlice(usize),
    SendParametersSlice,
    Delay,
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SendCommand {
    Nop,
    Default(&'static Command),
    // first usize is the position in the buffer
    // second usize is the length in the buffer starting from the position
    Position(&'static Command, usize, usize),
    // first usize is the position in the buffer (4 bytes - repeat times, length bytes data)
    // second usize is the length in the buffer
    // third usize is the number of repeats
    Repeat(&'static Command, usize, usize, usize),
    // usize is length
    Slice(&'static Command, usize),
}

pub struct ST7789H2<'a, A: Alarm<'a>> {
    mem: &'a dyn memory_async::Memory,
    alarm: &'a A,
    reset: &'a dyn gpio::Pin,
    status: Cell<Status>,
    callback: OptionalCell<Callback>,
    width: Cell<usize>,
    height: Cell<usize>,

    client: OptionalCell<&'static dyn screen::ScreenClient>,
    setup_client: OptionalCell<&'static dyn screen::ScreenSetupClient>,
    setup_command: Cell<bool>,

    sequence_buffer: TakeCell<'static, [SendCommand]>,
    position_in_sequence: Cell<usize>,
    sequence_len: Cell<usize>,
    command: Cell<&'static Command>,
    buffer: TakeCell<'static, [u8]>,

    power_on: Cell<bool>,

    write_buffer: TakeCell<'static, [u8]>,
}

impl<'a, A: Alarm<'a>> ST7789H2<'a, A> {
    pub fn new(
        mem: &'a dyn memory_async::Memory,
        alarm: &'a A,
        reset: &'a dyn gpio::Pin,
        buffer: &'static mut [u8],
        sequence_buffer: &'static mut [SendCommand],
    ) -> ST7789H2<'a, A> {
        reset.make_output();
        ST7789H2 {
            alarm: alarm,

            reset: reset,
            mem: mem,

            callback: OptionalCell::empty(),

            status: Cell::new(Status::Idle),
            width: Cell::new(240),
            height: Cell::new(240),

            client: OptionalCell::empty(),
            setup_client: OptionalCell::empty(),
            setup_command: Cell::new(false),

            sequence_buffer: TakeCell::new(sequence_buffer),
            sequence_len: Cell::new(0),
            position_in_sequence: Cell::new(0),
            command: Cell::new(&NOP),
            buffer: TakeCell::new(buffer),

            power_on: Cell::new(false),

            write_buffer: TakeCell::empty(),
        }
    }

    fn send_sequence(&self, sequence: CommandSequence) -> ReturnCode {
        if self.status.get() == Status::Idle {
            let error = self.sequence_buffer.map_or_else(
                || panic!("st7789h2: send sequence has no sequence buffer"),
                |sequence_buffer| {
                    if sequence.len() <= sequence_buffer.len() {
                        self.sequence_len.set(sequence.len());
                        for (i, cmd) in sequence.iter().enumerate() {
                            sequence_buffer[i] = *cmd;
                        }
                        ReturnCode::SUCCESS
                    } else {
                        ReturnCode::ENOMEM
                    }
                },
            );
            if error == ReturnCode::SUCCESS {
                self.send_sequence_buffer()
            } else {
                error
            }
        } else {
            ReturnCode::EBUSY
        }
    }

    fn send_sequence_buffer(&self) -> ReturnCode {
        if self.status.get() == Status::Idle {
            self.position_in_sequence.set(0);
            // set status to delay so that do_next_op will send the next item in the sequence
            self.status.set(Status::Delay);
            self.do_next_op();
            ReturnCode::SUCCESS
        } else {
            ReturnCode::EBUSY
        }
    }

    fn send_command_with_default_parameters(&self, cmd: &'static Command) {
        let mut len = 0;
        self.buffer.map_or_else(
            || panic!("st7789h2: send parameters has no buffer"),
            |buffer| {
                // buffer[0] = cmd.id;
                debug!("{} {:?}", cmd.id, cmd.parameters);
                if let Some(parameters) = cmd.parameters {
                    for parameter in parameters.iter() {
                        buffer[len] = *parameter;
                        len = len + 1;
                    }
                }
            },
        );
        self.send_command(cmd, 0, len, 1);
    }

    fn send_command(&self, cmd: &'static Command, position: usize, len: usize, repeat: usize) {
        self.command.set(cmd);
        self.status.set(Status::SendCommand(position, len, repeat));
        self.buffer.take().map_or_else(
            || panic!("st7789h2: send command has no buffer"),
            |buffer| {
                // buffer[0] = cmd.id;
                self.mem
                    .write_addr(BusWidth::Bits8, cmd.id as usize, BusWidth::Bits8, buffer, 0);
            },
        );
    }

    fn send_command_slice(&self, cmd: &'static Command, len: usize) {
        self.command.set(cmd);
        self.status.set(Status::SendCommandSlice(len));
        self.buffer.take().map_or_else(
            || panic!("st7789h2: send command has no buffer"),
            |buffer| {
                // buffer[0] = cmd.id;
                self.mem
                    .write_addr(BusWidth::Bits8, cmd.id as usize, BusWidth::Bits8, buffer, 0);
            },
        );
    }

    fn send_parameters(&self, position: usize, len: usize, repeat: usize) {
        self.status.set(Status::SendCommand(0, len, repeat - 1));
        if len > 0 {
            self.buffer.take().map_or_else(
                || panic!("st7789h2: send parameters has no buffer"),
                |buffer| {
                    // shift parameters
                    if position > 0 {
                        for i in position..len + position {
                            buffer[i - position] = buffer[i];
                        }
                    }
                    self.mem.write(BusWidth::Bits8, buffer, len);
                },
            );
        } else {
            self.do_next_op();
        }
    }

    fn send_parameters_slice(&self, len: usize) {
        self.write_buffer.take().map_or_else(
            || panic!("st7789h2: no write buffer"),
            |buffer| {
                self.status.set(Status::SendParametersSlice);
                self.mem.write(BusWidth::Bits16, buffer, len / 2);
            },
        );
    }

    fn fill(&self, color: usize) -> ReturnCode {
        if self.status.get() == Status::Idle {
            // TODO check if buffer is available
            self.sequence_buffer.map_or_else(
                || panic!("st7789h2: fill has no sequence buffer"),
                |sequence| {
                    sequence[0] = SendCommand::Default(&CASET);
                    sequence[1] = SendCommand::Default(&RASET);
                    self.buffer.map_or_else(
                        || panic!("st7789h2: fill has no buffer"),
                        |buffer| {
                            let bytes = 240 * 240 * 2;
                            let buffer_space = (buffer.len() - 9) / 2 * 2;
                            let repeat = (bytes / buffer_space) + 1;
                            sequence[2] = SendCommand::Repeat(&WRITE_RAM, 9, buffer_space, repeat);
                            for index in 0..(buffer_space / 2) {
                                buffer[9 + 2 * index] = ((color >> 8) & 0xFF) as u8;
                                buffer[9 + (2 * index + 1)] = color as u8;
                            }
                        },
                    );
                    self.sequence_len.set(3);
                },
            );
            self.send_sequence_buffer();
            ReturnCode::SUCCESS
        } else {
            ReturnCode::EBUSY
        }
    }

    fn rotation(&self, rotation: ScreenRotation) -> ReturnCode {
        // if self.status.get() == Status::Idle {
        //     let rotation_bits = match rotation {
        //         ScreenRotation::Normal => 0x00,
        //         ScreenRotation::Rotated90 => 0x60,
        //         ScreenRotation::Rotated180 => 0xC0,
        //         ScreenRotation::Rotated270 => 0xA0,
        //     };
        //     match rotation {
        //         ScreenRotation::Normal | ScreenRotation::Rotated180 => {
        //             self.width.set(128);
        //             self.height.set(160);
        //         }
        //         ScreenRotation::Rotated90 | ScreenRotation::Rotated270 => {
        //             self.width.set(160);
        //             self.height.set(128);
        //         }
        //     };
        //     self.buffer.map_or_else(
        //         || panic!("st7789h2: set rotation has no buffer"),
        //         |buffer| {
        //             buffer[1] =
        //                 rotation_bits | MADCTL.parameters.map_or(0, |parameters| parameters[0])
        //         },
        //     );
        //     self.setup_command.set(true);
        //     self.send_command(&MADCTL, 1, 1, 1);
        //     ReturnCode::SUCCESS
        // } else {
        //     ReturnCode::EBUSY
        // }
        self.send_command_with_default_parameters(&NOP);
        match rotation {
            ScreenRotation::Normal => ReturnCode::SUCCESS,
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn display_on(&self) -> ReturnCode {
        if self.status.get() == Status::Idle {
            if !self.power_on.get() {
                ReturnCode::EOFF
            } else {
                self.setup_command.set(false);
                self.send_command_with_default_parameters(&DISPLAY_ON);
                ReturnCode::SUCCESS
            }
        } else {
            ReturnCode::EBUSY
        }
    }

    fn display_off(&self) -> ReturnCode {
        if self.status.get() == Status::Idle {
            if !self.power_on.get() {
                ReturnCode::EOFF
            } else {
                self.setup_command.set(false);
                self.send_command_with_default_parameters(&DISPLAY_OFF);
                ReturnCode::SUCCESS
            }
        } else {
            ReturnCode::EBUSY
        }
    }

    fn display_invert_on(&self) -> ReturnCode {
        // if self.status.get() == Status::Idle {
        //     if !self.power_on.get() {
        //         ReturnCode::EOFF
        //     } else {
        //         self.setup_command.set(false);
        //         self.send_command_with_default_parameters(&INVON);
        //         ReturnCode::SUCCESS
        //     }
        // } else {
        //     ReturnCode::EBUSY
        // }
        ReturnCode::ENOSUPPORT
    }

    fn display_invert_off(&self) -> ReturnCode {
        // if self.status.get() == Status::Idle {
        //     if !self.power_on.get() {
        //         ReturnCode::EOFF
        //     } else {
        //         self.setup_command.set(false);
        //         self.send_command_with_default_parameters(&INVOFF);
        //         ReturnCode::SUCCESS
        //     }
        // } else {
        //     ReturnCode::EBUSY
        // }
        ReturnCode::ENOSUPPORT
    }

    fn do_next_op(&self) {
        match self.status.get() {
            Status::Delay => {
                self.sequence_buffer.map_or_else(
                    || panic!("st7789h2: do next op has no sequence buffer"),
                    |sequence| {
                        // debug!("enter sequence");
                        // sendf next command in the sequence
                        let position = self.position_in_sequence.get();
                        self.position_in_sequence
                            .set(self.position_in_sequence.get() + 1);
                        if position < self.sequence_len.get() {
                            debug!("{:?}", sequence[position]);
                            match sequence[position] {
                                SendCommand::Nop => {
                                    self.do_next_op();
                                }
                                SendCommand::Default(ref cmd) => {
                                    self.send_command_with_default_parameters(cmd);
                                }
                                SendCommand::Position(ref cmd, position, len) => {
                                    self.send_command(cmd, position, len, 1);
                                }
                                SendCommand::Repeat(ref cmd, position, len, repeat) => {
                                    self.send_command(cmd, position, len, repeat);
                                }
                                SendCommand::Slice(ref cmd, len) => {
                                    self.send_command_slice(cmd, len);
                                }
                            };
                        } else {
                            self.status.set(Status::Idle);
                            self.callback.map(|callback| {
                                callback.schedule(0, 0, 0);
                            });
                            if !self.power_on.get() {
                                self.client.map(|client| {
                                    self.power_on.set(true);
                                    client.screen_is_ready();
                                });
                            } else {
                                if self.setup_command.get() {
                                    self.setup_command.set(false);
                                    self.setup_client.map(|setup_client| {
                                        setup_client.command_complete(ReturnCode::SUCCESS);
                                    });
                                } else {
                                    self.client.map(|client| {
                                        if self.write_buffer.is_some() {
                                            self.write_buffer.take().map(|buffer| {
                                                client.write_complete(buffer, ReturnCode::SUCCESS);
                                            });
                                        } else {
                                            client.command_complete(ReturnCode::SUCCESS);
                                        }
                                    });
                                }
                            }
                        }
                        // debug!("exit sequence");
                    },
                );
            }
            Status::SendCommand(parameters_position, parameters_length, repeat) => {
                if repeat == 0 {
                    let mut delay = self.command.get().delay as u32;
                    if delay > 0 {
                        if delay == 255 {
                            delay = 500;
                        }
                        self.set_delay(delay, Status::Delay)
                    } else {
                        self.status.set(Status::Delay);
                        self.do_next_op();
                    }
                } else {
                    self.send_parameters(parameters_position, parameters_length, repeat);
                }
            }
            Status::SendCommandSlice(len) => {
                self.send_parameters_slice(len);
            }
            Status::SendParametersSlice => {
                let mut delay = self.command.get().delay as u32;
                if delay > 0 {
                    if delay == 255 {
                        delay = 500;
                    }
                    self.set_delay(delay, Status::Delay)
                } else {
                    self.status.set(Status::Delay);
                    self.do_next_op();
                }
            }
            Status::Reset1 => {
                self.send_command_with_default_parameters(&NOP);
                self.reset.clear();
                self.set_delay(5, Status::Reset2);
            }
            Status::Reset2 => {
                self.reset.set();
                self.set_delay(10, Status::Reset3);
            }
            Status::Reset3 => {
                self.reset.clear();
                self.set_delay(20, Status::Reset4);
            }
            Status::Reset4 => {
                self.reset.set();
                self.set_delay(10, Status::Init);
            }
            Status::Init => {
                self.status.set(Status::Idle);
                self.send_sequence(&INIT_SEQUENCE);
            }
            _ => {
                panic!("ST7789H2 status Idle");
            }
        };
    }

    fn set_memory_frame(
        &self,
        position: usize,
        sx: usize,
        sy: usize,
        ex: usize,
        ey: usize,
    ) -> ReturnCode {
        if sx <= self.width.get()
            && sy <= self.height.get()
            && ex <= self.width.get()
            && ey <= self.height.get()
            && sx <= ex
            && sy <= ey
        {
            if self.status.get() == Status::Idle {
                self.buffer.map_or_else(
                    || panic!("st7789h2: set memory frame has no buffer"),
                    |buffer| {
                        // CASET
                        buffer[position] = 0;
                        buffer[position + 1] = sx as u8;
                        buffer[position + 2] = 0;
                        buffer[position + 3] = ex as u8;
                        // RASET
                        buffer[position + 4] = 0;
                        buffer[position + 5] = sy as u8;
                        buffer[position + 6] = 0;
                        buffer[position + 7] = ey as u8;
                    },
                );
                ReturnCode::SUCCESS
            } else {
                ReturnCode::EBUSY
            }
        } else {
            ReturnCode::EINVAL
        }
    }

    // fn write_data(&self, data: &'static [u8], len: usize) -> ReturnCode {
    //     if self.status.get() == Status::Idle {
    //         self.buffer.map(|buffer| {
    //             // TODO verify length
    //             for position in 0..len {
    //                 buffer[position + 1] = data[position];
    //             }
    //         });
    //         self.send_command(&RAMWR, 1, len, 1);
    //         ReturnCode::SUCCESS
    //     } else {
    //         ReturnCode::EBUSY
    //     }
    // }

    fn write_pixel(&self, x: usize, y: usize, color: usize) -> ReturnCode {
        if x < self.width.get() && y < self.height.get() {
            if self.status.get() == Status::Idle {
                self.buffer.map_or_else(
                    || panic!("st7789h2: write pixel has no buffer"),
                    |buffer| {
                        // CASET
                        buffer[1] = 0;
                        buffer[2] = x as u8;
                        buffer[3] = 0;
                        buffer[4] = (x + 1) as u8;
                        // RASET
                        buffer[5] = 0;
                        buffer[6] = y as u8;
                        buffer[7] = 0;
                        buffer[8] = (y + 1) as u8;
                        // WRITE_RAM
                        buffer[9] = ((color >> 8) & 0xFF) as u8;
                        buffer[10] = (color & 0xFF) as u8
                    },
                );
                self.send_sequence(&WRITE_PIXEL)
            } else {
                ReturnCode::EBUSY
            }
        } else {
            ReturnCode::EINVAL
        }
    }

    pub fn init(&self) -> ReturnCode {
        if self.status.get() == Status::Idle {
            self.status.set(Status::Reset1);
            self.do_next_op();
            ReturnCode::SUCCESS
        } else {
            ReturnCode::EBUSY
        }
    }

    /// set_delay sets an alarm and saved the next state after that.
    ///
    /// As argument, there are:
    ///  - the duration of the alarm in ms
    ///  - the status of the program after the alarm fires
    ///
    /// Example:
    ///  self.set_delay(10, Status::Idle);
    fn set_delay(&self, timer: u32, next_status: Status) {
        self.status.set(next_status);
        self.alarm.set_alarm(
            self.alarm
                .now()
                .wrapping_add(<A::Frequency>::frequency() / 1000 * timer),
        )
    }
}

impl<'a, A: Alarm<'a>> Driver for ST7789H2<'a, A> {
    fn command(&self, command_num: usize, data1: usize, data2: usize, _: AppId) -> ReturnCode {
        match command_num {
            0 => ReturnCode::SUCCESS,
            // reset
            1 => self.init(),
            // fill with color (data1)
            2 => self.fill(data1),
            // write pixel (x:data1[15:8], y:data1[7:0], color:data2)
            3 => self.write_pixel((data1 >> 8) & 0xFF, data1 & 0xFF, data2),
            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        _app_id: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            0 => {
                self.callback.insert(callback);
                ReturnCode::SUCCESS
            }
            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

impl<'a, A: Alarm<'a>> screen::ScreenSetup for ST7789H2<'a, A> {
    fn set_client(&self, setup_client: Option<&'static dyn ScreenSetupClient>) {
        if let Some(setup_client) = setup_client {
            self.setup_client.set(setup_client);
        } else {
            self.setup_client.clear();
        }
    }

    fn set_resolution(&self, resolution: (usize, usize)) -> ReturnCode {
        if self.status.get() == Status::Idle {
            if resolution.0 == self.width.get() && resolution.1 == self.height.get() {
                self.setup_client
                    .map(|setup_client| setup_client.command_complete(ReturnCode::SUCCESS));
                ReturnCode::SUCCESS
            } else {
                ReturnCode::ENOSUPPORT
            }
        } else {
            ReturnCode::EBUSY
        }
    }

    fn set_pixel_format(&self, depth: ScreenPixelFormat) -> ReturnCode {
        if self.status.get() == Status::Idle {
            if depth == ScreenPixelFormat::RGB_565 {
                self.setup_client
                    .map(|setup_client| setup_client.command_complete(ReturnCode::SUCCESS));
                ReturnCode::SUCCESS
            } else {
                ReturnCode::EINVAL
            }
        } else {
            ReturnCode::EBUSY
        }
    }

    fn set_rotation(&self, rotation: ScreenRotation) -> ReturnCode {
        self.rotation(rotation)
    }

    fn get_num_supported_resolutions(&self) -> usize {
        1
    }
    fn get_supported_resolution(&self, index: usize) -> Option<(usize, usize)> {
        match index {
            0 => Some((self.width.get(), self.height.get())),
            _ => None,
        }
    }

    fn get_num_supported_pixel_formats(&self) -> usize {
        1
    }
    fn get_supported_pixel_format(&self, index: usize) -> Option<ScreenPixelFormat> {
        match index {
            0 => Some(ScreenPixelFormat::RGB_565),
            _ => None,
        }
    }
}

impl<'a, A: Alarm<'a>> screen::Screen for ST7789H2<'a, A> {
    fn get_resolution(&self) -> (usize, usize) {
        (self.width.get(), self.height.get())
    }

    fn get_pixel_format(&self) -> ScreenPixelFormat {
        ScreenPixelFormat::RGB_565
    }

    fn get_rotation(&self) -> ScreenRotation {
        ScreenRotation::Normal
    }

    fn set_write_frame(&self, x: usize, y: usize, width: usize, height: usize) -> ReturnCode {
        if self.status.get() == Status::Idle {
            self.setup_command.set(false);
            let buffer_len = self.buffer.map_or_else(
                || panic!("st7789h2: buffer is not available"),
                |buffer| buffer.len() - 1,
            );
            if buffer_len >= 9 {
                // set buffer
                let err = self.set_memory_frame(0, x, y, x + width - 1, y + height - 1);
                if err == ReturnCode::SUCCESS {
                    self.sequence_buffer.map_or_else(
                        || panic!("st7789h2: write no sequence buffer"),
                        |sequence| {
                            sequence[0] = SendCommand::Position(&CASET, 0, 4);
                            sequence[1] = SendCommand::Position(&RASET, 4, 4);
                            self.sequence_len.set(2);
                        },
                    );
                    self.send_sequence_buffer();
                }
                err
            } else {
                ReturnCode::ENOMEM
            }
        } else {
            ReturnCode::EBUSY
        }
    }

    fn write(&self, buffer: &'static mut [u8], len: usize) -> ReturnCode {
        if self.status.get() == Status::Idle {
            self.setup_command.set(false);
            self.write_buffer.replace(buffer);
            let buffer_len = self.buffer.map_or_else(
                || panic!("st7789h2: buffer is not available"),
                |buffer| buffer.len(),
            );
            if buffer_len > 0 {
                // set buffer
                self.sequence_buffer.map_or_else(
                    || panic!("st7789h2: write no sequence buffer"),
                    |sequence| {
                        sequence[0] = SendCommand::Slice(&WRITE_RAM, len);
                        self.sequence_len.set(1);
                    },
                );
                self.send_sequence_buffer();
                ReturnCode::SUCCESS
            } else {
                ReturnCode::ENOMEM
            }
        } else {
            ReturnCode::EBUSY
        }
    }

    fn set_client(&self, client: Option<&'static dyn ScreenClient>) {
        if let Some(client) = client {
            self.client.set(client);
        } else {
            self.client.clear();
        }
    }

    fn set_brightness(&self, brightness: usize) -> ReturnCode {
        if brightness > 0 {
            self.display_on()
        } else {
            self.display_off()
        }
    }

    fn invert_on(&self) -> ReturnCode {
        self.display_invert_on()
    }

    fn invert_off(&self) -> ReturnCode {
        self.display_invert_off()
    }
}

impl<'a, A: Alarm<'a>> time::AlarmClient for ST7789H2<'a, A> {
    fn fired(&self) {
        self.do_next_op();
    }
}

impl<'a, A: Alarm<'a>> memory_async::Client for ST7789H2<'a, A> {
    fn command_complete(&self, buffer: &'static mut [u8], _len: usize) {
        if self.status.get() == Status::SendParametersSlice {
            self.write_buffer.replace(buffer);
        } else {
            self.buffer.replace(buffer);
        }
        // debug!("do next op");
        self.do_next_op();
    }
}
