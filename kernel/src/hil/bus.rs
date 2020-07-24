use crate::returncode::ReturnCode;

pub enum Error {
    Complete,
    Error,
}

pub enum BusWidth {
    Bits8,
    Bits16LE,
    Bits16BE,
    Bits32LE,
    Bits32BE,
    Bits64LE,
    Bits64BE,
}

pub trait Bus {
    fn write_addr(
        &self,
        addr_width: BusWidth,
        addr: usize,
        data_width: BusWidth,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (ReturnCode, &'static mut [u8])>;
    fn read_addr(
        &self,
        addr_width: BusWidth,
        addr: usize,
        data_width: BusWidth,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (ReturnCode, &'static mut [u8])>;

    fn write(
        &self,
        data_width: BusWidth,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (ReturnCode, &'static mut [u8])>;
    fn read(
        &self,
        data_width: BusWidth,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (ReturnCode, &'static mut [u8])>;

    fn set_client(&self, client: &'static dyn Client);
}

pub trait Client {
    fn command_complete(&self, buffer: &'static mut [u8], len: usize);
}
