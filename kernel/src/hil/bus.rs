//! Abstraction Interface for several busses.
//! Useful for devices that support multiple protocols

use crate::returncode::ReturnCode;

pub enum BusWidth {
    Bits8,
    Bits16LE,
    Bits16BE,
    Bits32LE,
    Bits32BE,
    Bits64LE,
    Bits64BE,
}

impl BusWidth {
    pub fn width_in_bytes(&self) -> usize {
        match self {
            BusWidth::Bits8 => 1,
            BusWidth::Bits16BE | BusWidth::Bits16LE => 2,
            BusWidth::Bits32BE | BusWidth::Bits32LE => 3,
            BusWidth::Bits64BE | BusWidth::Bits64LE => 4,
        }
    }
}

pub trait Bus {
    fn set_addr(&self, addr_width: BusWidth, addr: usize) -> ReturnCode;

    fn write(&self, data_width: BusWidth, buffer: &'static mut [u8], len: usize) -> ReturnCode;
    fn read(&self, data_width: BusWidth, buffer: &'static mut [u8], len: usize) -> ReturnCode;

    fn set_client(&self, client: &'static dyn Client);
}

pub trait Client {
    fn command_complete(&self, buffer: Option<&'static mut [u8]>, len: usize);
}
