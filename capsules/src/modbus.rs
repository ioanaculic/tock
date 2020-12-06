pub enum ModBusError {

}

pub trait ModBusTransportClient {
    fn packet_received (&self, buffer: &'static [u8], len: usize);
    fn packet_sent (&self, buffer: &'static [u8], error: ModBusError);
}

pub trait ModBusTransport<'a> {
    fn send_packet (&self, buffer: &'static [u8], len: usize);
    fn set_client (&self, client: &'a dyn ModBusTransportClient);
}

pub struct ModBusServer<'a, T: ModBusTransport<'a>> {
    transport: &'a T
}

impl <'a, T: ModBusTransport<'a>> ModBusServer<'a, T> {
    pub fn new (transport: &'a T) -> ModBusServer<'a, T> {
        ModBusServer {
            transport
        }
    }
}
