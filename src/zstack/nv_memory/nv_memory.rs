use crate::{subscription::SubscriptionService, zstack::unpi::{commands::Command, serial::{wait_for, UnpiCommandError}, MessageType, SUnpiPacket, Subsystem}};
use futures::lock::Mutex;
use std::sync::Arc;

pub struct NvMemoryAdapter {
    subscriptions: Arc<Mutex<SubscriptionService<SUnpiPacket>>>,
}

impl NvMemoryAdapter {
    pub fn new(
        subscriptions: Arc<Mutex<SubscriptionService<SUnpiPacket>>>,
    ) -> Result<Self, NvMemoryAdapterError> {
        Ok(NvMemoryAdapter { subscriptions })
    }

    async fn wait_for(
        &self,
        name: &str,
        message_type: MessageType,
        subsystem: Subsystem,
        timeout: Option<std::time::Duration>,
    ) -> Result<(SUnpiPacket, Command), NvMemoryAdapterError> {
        Ok(wait_for(
            name,
            message_type,
            subsystem,
            self.subscriptions.clone(),
            timeout,
        )
        .await?)
    }

    pub fn read_item(&self, id: u16) -> Result<Vec<u8>, NvMemoryAdapterError> {
        let (packet, _) = futures::executor::block_on(self.wait_for(
            "NV_READ",
            MessageType::AREQ,
            Subsystem::Sys,
            Some(std::time::Duration::from_secs(5)),
        ))?;
        if packet.payload.len() < 2 {
            return Err(NvMemoryAdapterError::InvalidData);
        }
        let item_id = u16::from_le_bytes([packet.payload[0], packet.payload[1]]);
        if item_id != id {
            return Err(NvMemoryAdapterError::InvalidData);
        }
        Ok(packet.payload[2..].to_vec())
    }
}

#[derive(Debug)]
pub enum NvMemoryAdapterError {
    IoError(std::io::Error),
    InvalidData,
    UnpiCommand(UnpiCommandError),
}

impl From<UnpiCommandError> for NvMemoryAdapterError {
    fn from(e: UnpiCommandError) -> Self {
        NvMemoryAdapterError::UnpiCommand(e)
    }
}
