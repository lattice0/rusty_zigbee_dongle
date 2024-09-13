use crate::coordinator::CoordinatorError;
use std::future::Future;

pub mod simple_serial_port;

pub trait SubscriptionSerial<P> {
    type Sender;
    type Receiver;

    fn write(&mut self, packet: &P) -> impl Future<Output = Result<(), CoordinatorError>>;
    fn start(&mut self) -> Result<(), CoordinatorError>;
}

#[derive(Debug, PartialEq)]
pub enum SerialThreadError {
    SerialOpen,
    SerialRead(String),
    SerialWrite(String),
    MalformedPacket,
    SubscriptionWrite,
    PacketParse,
}
