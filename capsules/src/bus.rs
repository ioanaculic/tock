use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::debug;
use kernel::hil::bus::{Bus, BusWidth, Client};
use kernel::hil::i2c::{Error, I2CClient, I2CDevice};
use kernel::hil::spi::{ClockPhase, ClockPolarity, SpiMasterClient, SpiMasterDevice};
use kernel::ReturnCode;

fn bus_width_in_bytes(bus_width: &BusWidth) -> usize {
    match bus_width {
        BusWidth::Bits8 => 1,
        BusWidth::Bits16BE | BusWidth::Bits16LE => 2,
        BusWidth::Bits32BE | BusWidth::Bits32LE => 3,
        BusWidth::Bits64BE | BusWidth::Bits64LE => 4,
    }
}

#[derive(Copy, Clone)]
enum BusStatus {
    Idle,
    SetAddress,
    Write,
    Read,
}

/*********** SPI ************/

pub struct SpiBus<'a> {
    spi: &'a dyn SpiMasterDevice,
    read_write_buffer: OptionalCell<&'static mut [u8]>,
    bus_width: Cell<usize>,
    client: OptionalCell<&'a dyn Client>,
    addr_buffer: OptionalCell<&'static mut [u8]>,
    status: Cell<BusStatus>,
}

impl<'a> SpiBus<'a> {
    pub fn new(spi: &'a dyn SpiMasterDevice, addr_buffer: &'static mut [u8]) -> SpiBus<'a> {
        SpiBus {
            spi,
            read_write_buffer: OptionalCell::empty(),
            bus_width: Cell::new(1),
            client: OptionalCell::empty(),
            addr_buffer: OptionalCell::new(addr_buffer),
            status: Cell::new(BusStatus::Idle),
        }
    }

    pub fn set_read_write_buffer(&self, buffer: &'static mut [u8]) {
        self.read_write_buffer.replace(buffer);
    }

    pub fn configure(&self, cpol: ClockPolarity, cpal: ClockPhase, rate: u32) {
        self.spi.configure(cpol, cpal, rate);
    }
}

impl<'a> Bus for SpiBus<'a> {
    fn set_addr(&self, addr_width: BusWidth, addr: usize) -> ReturnCode {
        match addr_width {
            BusWidth::Bits8 => self
                .addr_buffer
                .take()
                .map_or(ReturnCode::ENOMEM, |buffer| {
                    self.status.set(BusStatus::SetAddress);
                    buffer[0] = addr as u8;
                    self.spi.read_write_bytes(buffer, None, 1);
                    ReturnCode::SUCCESS
                }),

            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn write(&self, data_width: BusWidth, buffer: &'static mut [u8], len: usize) -> ReturnCode {
        // endianess does not matter as the buffer is sent as is
        let bytes = bus_width_in_bytes(&data_width);
        self.bus_width.set(bytes);
        if buffer.len() >= len * bytes {
            self.status.set(BusStatus::Write);
            self.spi.read_write_bytes(buffer, None, len * bytes);
            ReturnCode::SUCCESS
        } else {
            ReturnCode::ENOMEM
        }
    }

    fn read(&self, data_width: BusWidth, buffer: &'static mut [u8], len: usize) -> ReturnCode {
        // endianess does not matter as the buffer is read as is
        let bytes = bus_width_in_bytes(&data_width);
        self.bus_width.set(bytes);
        self.read_write_buffer.take().map_or_else(
            || panic!("bus::read: spi did not return the read write buffer"),
            move |write_buffer| {
                if write_buffer.len() >= len * bytes
                    && write_buffer.len() > 0
                    && buffer.len() > len * bytes
                {
                    self.status.set(BusStatus::Read);
                    self.spi
                        .read_write_bytes(write_buffer, Some(buffer), len * bytes);
                    ReturnCode::SUCCESS
                } else {
                    ReturnCode::ENOMEM
                }
            },
        )
    }

    fn set_client(&self, client: &'static dyn Client) {
        self.client.replace(client);
    }
}

impl<'a> SpiMasterClient for SpiBus<'a> {
    fn read_write_done(
        &self,
        write_buffer: &'static mut [u8],
        read_buffer: Option<&'static mut [u8]>,
        len: usize,
    ) {
        // debug!("write done {}", len);
        match self.status.get() {
            BusStatus::SetAddress => {
                self.addr_buffer.replace(write_buffer);
                self.client
                    .map(move |client| client.command_complete(None, 0));
            }
            BusStatus::Write | BusStatus::Read => {
                let mut buffer = write_buffer;
                if let Some(buf) = read_buffer {
                    self.read_write_buffer.replace(buffer);
                    buffer = buf;
                }
                self.client.map(move |client| {
                    client.command_complete(Some(buffer), len / self.bus_width.get())
                });
            }
            _ => {
                panic!("spi sent an extra read_write_done");
            }
        }
    }
}

/*********** I2C ************/

pub struct I2CBus<'a> {
    i2c: &'a dyn I2CDevice,
    len: Cell<usize>,
    client: OptionalCell<&'a dyn Client>,
    addr_buffer: OptionalCell<&'static mut [u8]>,
    status: Cell<BusStatus>,
}

impl<'a> I2CBus<'a> {
    pub fn new(i2c: &'a dyn I2CDevice) -> I2CBus {
        I2CBus {
            i2c,
            len: Cell::new(0),
            client: OptionalCell::empty(),
            addr_buffer: OptionalCell::empty(),
            status: Cell::new(BusStatus::Idle),
        }
    }
}

impl<'a> Bus for I2CBus<'a> {
    fn set_addr(&self, addr_width: BusWidth, addr: usize) -> ReturnCode {
        match addr_width {
            BusWidth::Bits8 => self
                .addr_buffer
                .take()
                .map_or(ReturnCode::ENOMEM, |buffer| {
                    buffer[0] = addr as u8;
                    self.status.set(BusStatus::SetAddress);
                    self.i2c.write(buffer, 1);
                    ReturnCode::SUCCESS
                }),

            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn write(&self, data_width: BusWidth, buffer: &'static mut [u8], len: usize) -> ReturnCode {
        // endianess does not matter as the buffer is sent as is
        let bytes = bus_width_in_bytes(&data_width);
        self.len.set(len * bytes);
        if len * bytes < 255 && buffer.len() >= len * bytes {
            debug!("write len {}", len);
            self.len.set(len);
            self.status.set(BusStatus::Write);
            self.i2c.write(buffer, (len * bytes) as u8);
            ReturnCode::SUCCESS
        } else {
            ReturnCode::ENOMEM
        }
    }

    fn read(&self, data_width: BusWidth, buffer: &'static mut [u8], len: usize) -> ReturnCode {
        // endianess does not matter as the buffer is read as is
        let bytes = bus_width_in_bytes(&data_width);
        self.len.set(len * bytes);
        if len & bytes < 255 && buffer.len() >= len * bytes {
            self.len.set(len);
            self.status.set(BusStatus::Read);
            self.i2c.read(buffer, (len * bytes) as u8);
            ReturnCode::SUCCESS
        } else {
            ReturnCode::ENOMEM
        }
    }

    fn set_client(&self, client: &'static dyn Client) {
        self.client.replace(client);
    }
}

impl<'a> I2CClient for I2CBus<'a> {
    fn command_complete(&self, buffer: &'static mut [u8], error: Error) {
        let len = match error {
            Error::CommandComplete => self.len.get(),
            _ => 0,
        };
        match self.status.get() {
            BusStatus::SetAddress => {
                self.addr_buffer.replace(buffer);
                self.client
                    .map(move |client| client.command_complete(None, 0));
            }
            BusStatus::Write | BusStatus::Read => {
                self.client
                    .map(move |client| client.command_complete(Some(buffer), len));
            }
            _ => {
                panic!("i2c sent an extra read_write_done");
            }
        }
    }
}
