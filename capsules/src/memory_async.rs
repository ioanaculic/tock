use kernel::common::cells::OptionalCell;
use kernel::hil::memory_async::{Client, Memory};
use kernel::hil::spi::{SpiMasterClient, SpiMasterDevice};
use kernel::ReturnCode;

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
    fn write_addr_8(&self, addr: u8, buffer: &'static mut [u8], len: usize) -> ReturnCode {
        if buffer.len() > len {
            for index in (0..len).rev() {
                buffer[index + 1] = buffer[index];
            }
            buffer[0] = addr;
            self.write(buffer, len)
        } else {
            self.client
                .map(move |client| client.command_complete(buffer, 0));
            ReturnCode::ENOMEM
        }
    }

    fn read_addr_8(&self, addr: u8, buffer: &'static mut [u8], len: usize) -> ReturnCode {
        self.read_write_buffer
            .take()
            .map_or(ReturnCode::ENOMEM, move |write_buffer| {
                if write_buffer.len() > len && write_buffer.len() > 0 && buffer.len() > len {
                    write_buffer[0] = addr;
                    self.read_write_buffer.replace(write_buffer);
                    self.read(buffer, len)
                } else {
                    ReturnCode::ENOMEM
                }
            })
    }

    fn write(&self, buffer: &'static mut [u8], len: usize) -> ReturnCode {
        if buffer.len() >= len {
            self.spi.read_write_bytes(buffer, None, len)
        } else {
            self.client
                .map(move |client| client.command_complete(buffer, 0));
            ReturnCode::ENOMEM
        }
    }

    fn read(&self, buffer: &'static mut [u8], len: usize) -> ReturnCode {
        self.read_write_buffer
            .take()
            .map_or(ReturnCode::ENOMEM, move |write_buffer| {
                if write_buffer.len() >= len && write_buffer.len() > 0 && buffer.len() > len {
                    self.spi.read_write_bytes(write_buffer, Some(buffer), len)
                } else {
                    ReturnCode::ENOMEM
                }
            })
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
        let mut buffer = write_buffer;
        if let Some(buf) = read_buffer {
            self.read_write_buffer.replace(buffer);
            buffer = buf;
        }
        self.client
            .map(move |client| client.command_complete(buffer, len));
    }
}
