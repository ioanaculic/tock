use kernel::common::cells::OptionalCell;
use kernel::debug;
use kernel::hil::memory_async::{BusWidth, Client, Memory};
use kernel::hil::spi::{SpiMasterClient, SpiMasterDevice};
use kernel::ReturnCode;

fn bus_width_in_bytes(bus_width: BusWidth) -> usize {
    match bus_width {
        BusWidth::Bits8 => 1,
        BusWidth::Bits16 => 2,
        BusWidth::Bits32 => 3,
        BusWidth::Bits64 => 4,
    }
}

pub struct SpiMemory<'a> {
    spi: &'a dyn SpiMasterDevice,
    read_write_buffer: OptionalCell<&'static mut [u8]>,
    client: OptionalCell<&'a dyn Client>,
}

impl<'a> SpiMemory<'a> {
    pub fn new(spi: &'a dyn SpiMasterDevice) -> SpiMemory {
        SpiMemory {
            spi,
            read_write_buffer: OptionalCell::empty(),
            client: OptionalCell::empty(),
        }
    }

    pub fn set_read_write_buffer(&self, buffer: &'static mut [u8]) {
        self.read_write_buffer.replace(buffer);
    }
}

impl<'a> Memory for SpiMemory<'a> {
    fn write_addr(
        &self,
        addr_width: BusWidth,
        addr: usize,
        data_width: BusWidth,
        buffer: &'static mut [u8],
        len: usize,
    ) -> ReturnCode {
        match addr_width {
            BusWidth::Bits8 => {
                if buffer.len() > len {
                    for index in (0..len).rev() {
                        buffer[index + 1] = buffer[index];
                    }
                    buffer[0] = addr as u8;
                    self.write(data_width, buffer, len)
                } else {
                    ReturnCode::ENOMEM
                }
            }

            _ => ReturnCode::ENOSUPPORT,
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

    fn read_addr(
        &self,
        addr_width: BusWidth,
        addr: usize,
        data_width: BusWidth,
        buffer: &'static mut [u8],
        len: usize,
    ) -> ReturnCode {
        match addr_width {
            BusWidth::Bits8 => {
                self.read_write_buffer
                    .take()
                    .map_or(ReturnCode::ENOMEM, move |write_buffer| {
                        if write_buffer.len() > len && write_buffer.len() > 0 && buffer.len() > len
                        {
                            write_buffer[0] = addr as u8;
                            self.read_write_buffer.replace(write_buffer);
                            self.read(data_width, buffer, len)
                        } else {
                            ReturnCode::ENOMEM
                        }
                    })
            }

            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn write(&self, data_width: BusWidth, buffer: &'static mut [u8], len: usize) -> ReturnCode {
        match data_width {
            BusWidth::Bits8 => {
                if buffer.len() >= len {
                    debug!("write len {}", len);
                    self.spi.read_write_bytes(buffer, None, len)
                } else {
                    // panic!("write error");
                    ReturnCode::ENOMEM
                }
            }
            _ => ReturnCode::ENOSUPPORT,
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

    fn read(&self, data_width: BusWidth, buffer: &'static mut [u8], len: usize) -> ReturnCode {
        match data_width {
            BusWidth::Bits8 => {
                self.read_write_buffer
                    .take()
                    .map_or(ReturnCode::ENOMEM, move |write_buffer| {
                        if write_buffer.len() >= len && write_buffer.len() > 0 && buffer.len() > len
                        {
                            self.spi.read_write_bytes(write_buffer, Some(buffer), len)
                        } else {
                            ReturnCode::ENOMEM
                        }
                    })
            }

            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn set_client(&self, client: &'static dyn Client) {
        self.client.replace(client);
    }
}

impl<'a> SpiMasterClient for SpiMemory<'a> {
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
