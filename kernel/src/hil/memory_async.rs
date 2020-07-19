use crate::returncode::ReturnCode;

pub enum Error {
    Complete,
    Error,
}

pub trait Memory {
    fn write_addr_8(&self, addr: u8, buffer: &'static mut [u8], len: usize) -> ReturnCode;
    fn read_addr_8(&self, addr: u8, buffer: &'static mut [u8], len: usize) -> ReturnCode;

    fn write(&self, buffer: &'static mut [u8], len: usize) -> ReturnCode;
    fn read(&self, buffer: &'static mut [u8], len: usize) -> ReturnCode;

    fn set_client(&self, client: &'static dyn Client);
}

pub trait Client {
    fn command_complete(&self, buffer: &'static mut [u8], len: usize);
}
