use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::debug;
use kernel::hil::bus::{Bus, BusWidth, Client};
use kernel::hil::i2c::{Error, I2CClient, I2CDevice};
use kernel::hil::spi::{SpiMasterClient, SpiMasterDevice};
use kernel::ReturnCode;

fn bus_width_in_bytes(bus_width: &BusWidth) -> usize {
    match bus_width {
        BusWidth::Bits8 => 1,
        BusWidth::Bits16BE | BusWidth::Bits16LE => 2,
        BusWidth::Bits32BE | BusWidth::Bits32LE => 3,
        BusWidth::Bits64BE | BusWidth::Bits64LE => 4,
    }
}

/*********** SPI ************/

pub struct SpiBus<'a> {
    spi: &'a dyn SpiMasterDevice,
    read_write_buffer: OptionalCell<&'static mut [u8]>,
    client: OptionalCell<&'a dyn Client>,
}

impl<'a> SpiBus<'a> {
    pub fn new(spi: &'a dyn SpiMasterDevice) -> SpiBus {
        SpiBus {
            spi,
            read_write_buffer: OptionalCell::empty(),
            client: OptionalCell::empty(),
        }
    }

    pub fn set_read_write_buffer(&self, buffer: &'static mut [u8]) {
        self.read_write_buffer.replace(buffer);
    }
}

impl<'a> Bus for SpiBus<'a> {
    fn write_addr(
        &self,
        addr_width: BusWidth,
        addr: usize,
        data_width: BusWidth,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (ReturnCode, &'static mut [u8])> {
        match addr_width {
            BusWidth::Bits8 => {
                if buffer.len() > len {
                    for index in (0..len).rev() {
                        buffer[index + 1] = buffer[index];
                    }
                    buffer[0] = addr as u8;
                    self.write(data_width, buffer, len)
                } else {
                    Err((ReturnCode::ENOMEM, buffer))
                }
            }

            _ => Err((ReturnCode::ENOSUPPORT, buffer)),
        }
    }

    fn read_addr(
        &self,
        addr_width: BusWidth,
        addr: usize,
        data_width: BusWidth,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (ReturnCode, &'static mut [u8])> {
        match addr_width {
            BusWidth::Bits8 => self.read_write_buffer.take().map_or_else(
                || panic! ("bus::read_addr: spi did not return the read write buffer"),
                move |write_buffer| {
                    if write_buffer.len() > len && write_buffer.len() > 0 && buffer.len() > len {
                        write_buffer[0] = addr as u8;
                        self.read_write_buffer.replace(write_buffer);
                        self.read(data_width, buffer, len)
                    } else {
                        Err((ReturnCode::ENOMEM, buffer))
                    }
                },
            ),
            _ => Err((ReturnCode::ENOSUPPORT, buffer)),
        }
    }

    fn write(
        &self,
        data_width: BusWidth,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (ReturnCode, &'static mut [u8])> {
        // endianess does not matter as the buffer is sent as is
        let bytes = bus_width_in_bytes(&data_width);
        if buffer.len() >= len * bytes {
            debug!("write len {}", len);
            match self.spi.read_write_bytes(buffer, None, len) {
                Err((code, buffer, _)) => Err((code, buffer)),
                Ok(()) => Ok(()),
            }
        } else {
            // panic!("write error");
            Err((ReturnCode::ENOMEM, buffer))
        }
        // match status {
        //     ReturnCode::SUCCESS | ReturnCode::SuccessWithValue{} => {},
        //     _ => {
        //         self.client
        //         .map(move |client| client.command_complete(buffer, 0));
        //     }
        // };
        // status
    }

    fn read(
        &self,
        data_width: BusWidth,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (ReturnCode, &'static mut [u8])> {
        // endianess does not matter as the buffer is read as is
        let bytes = bus_width_in_bytes(&data_width);
        self.read_write_buffer.take().map_or_else(
            || panic! ("bus::read: spi did not return the read write buffer"),
            move |write_buffer| {
                if write_buffer.len() >= len * bytes
                    && write_buffer.len() > 0
                    && buffer.len() > len * bytes
                {
                    match self.spi.read_write_bytes(write_buffer, Some(buffer), len) {
                        Err((code, write_buffer, buffer)) => {
                            if let Some(buffer) = buffer {
                                self.read_write_buffer.replace(write_buffer);
                                Err((code, buffer))
                            } else {
                                panic!("spi did not return the read buffer");
                            }
                        }
                        Ok(()) => Ok(()),
                    }
                } else {
                    Err((ReturnCode::ENOMEM, buffer))
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
        debug!("write done {}", len);
        let mut buffer = write_buffer;
        if let Some(buf) = read_buffer {
            self.read_write_buffer.replace(buffer);
            buffer = buf;
        }
        self.client
            .map(move |client| client.command_complete(buffer, len));
    }
}

/*********** I2C ************/

pub struct I2CBus<'a> {
    i2c: &'a dyn I2CDevice,
    len: Cell<usize>,
    client: OptionalCell<&'a dyn Client>,
}

impl<'a> I2CBus<'a> {
    pub fn new(i2c: &'a dyn I2CDevice) -> I2CBus {
        I2CBus {
            i2c,
            len: Cell::new(0),
            client: OptionalCell::empty(),
        }
    }
}

impl<'a> Bus for I2CBus<'a> {
    fn write_addr(
        &self,
        addr_width: BusWidth,
        addr: usize,
        data_width: BusWidth,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (ReturnCode, &'static mut [u8])> {
        match addr_width {
            BusWidth::Bits8 => {
                if buffer.len() > len {
                    for index in (0..len).rev() {
                        buffer[index + 1] = buffer[index];
                    }
                    buffer[0] = addr as u8;
                    self.write(data_width, buffer, len)
                } else {
                    Err((ReturnCode::ENOMEM, buffer))
                }
            }

            _ => Err((ReturnCode::ENOSUPPORT, buffer)),
        }
    }

    fn read_addr(
        &self,
        addr_width: BusWidth,
        addr: usize,
        _data_width: BusWidth,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (ReturnCode, &'static mut [u8])> {
        match addr_width {
            BusWidth::Bits8 => {
                if len < 255 && buffer.len() > len {
                    buffer[0] = addr as u8;
                    self.i2c.write_read(buffer, 1, len as u8)
                } else {
                    Err((ReturnCode::ENOMEM, buffer))
                }
            }
            _ => Err((ReturnCode::ENOSUPPORT, buffer)),
        }
    }

    fn write(
        &self,
        data_width: BusWidth,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (ReturnCode, &'static mut [u8])> {
        // endianess does not matter as the buffer is sent as is
        let bytes = bus_width_in_bytes(&data_width);
        if len * bytes < 255 && buffer.len() >= len * bytes {
            debug!("write len {}", len);
            self.len.set(len);
            self.i2c.write(buffer, (len * bytes) as u8)
        } else {
            Err((ReturnCode::ENOMEM, buffer))
        }
    }

    fn read(
        &self,
        data_width: BusWidth,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (ReturnCode, &'static mut [u8])> {
        // endianess does not matter as the buffer is read as is
        let bytes = bus_width_in_bytes(&data_width);
        if len & bytes < 255 && buffer.len() >= len * bytes {
            self.len.set(len);
            self.i2c.read(buffer, (len * bytes) as u8)
        } else {
            Err((ReturnCode::ENOMEM, buffer))
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
        self.client
            .map(move |client| client.command_complete(buffer, len));
    }
}
