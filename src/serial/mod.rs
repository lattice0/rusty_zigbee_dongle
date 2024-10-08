use std::future::Future;

pub mod simple_serial_port;

pub trait SimpleSerial<P> {
    type Sender;
    type Receiver;

    /// Writes to the serial port asynchonously via a channel
    fn write(&mut self, packet: &P) -> impl Future<Output = Result<(), SerialThreadError>>;
}

#[derive(Debug, PartialEq)]
pub enum SerialThreadError {
    SerialOpen,
    SerialRead(String),
    SerialWrite(String),
    MalformedPacket,
    SubscriptionWrite,
    PacketParse,
    SerialChannelMissing,
    SerialChannel,
}
