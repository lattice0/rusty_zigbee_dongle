use crate::{
    parameters::ParameterValue,
    serial::SimpleSerial,
    subscription::SubscriptionService,
    zstack::unpi::{
        commands::Command,
        serial::{request, wait_for, UnpiCommandError},
        MessageType, SUnpiPacket, Subsystem,
    },
};
use futures::lock::Mutex;
use std::sync::Arc;

pub struct NvMemoryAdapter<S: SimpleSerial<SUnpiPacket>> {
    serial: Arc<Mutex<S>>,
    subscriptions: Arc<Mutex<SubscriptionService<SUnpiPacket>>>,
}

impl<S: SimpleSerial<SUnpiPacket>> NvMemoryAdapter<S> {
    pub fn new(
        serial: Arc<Mutex<S>>,
        subscriptions: Arc<Mutex<SubscriptionService<SUnpiPacket>>>,
    ) -> Result<Self, NvMemoryAdapterError> {
        Ok(NvMemoryAdapter {
            serial,
            subscriptions,
        })
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

    async fn request(
        &self,
        name: &str,
        subsystem: Subsystem,
        parameters: &[(&'static str, ParameterValue)],
        _timeout: Option<std::time::Duration>,
    ) -> Result<(), NvMemoryAdapterError> {
        request(name, subsystem, parameters, self.serial.clone()).await?;
        Ok(())
    }

    async fn request_with_reply(
        &self,
        name: &str,
        message_type: MessageType,
        subsystem: Subsystem,
        parameters: &[(&'static str, ParameterValue)],
        timeout: Option<std::time::Duration>,
    ) -> Result<(), NvMemoryAdapterError> {
        self.wait_for(name, message_type, subsystem, timeout)
            .await?;
        request(name, subsystem, parameters, self.serial.clone()).await?;
        Ok(())
    }

    pub async fn read_item(&self, _id: u16) -> Result<Vec<u8>, NvMemoryAdapterError> {
        let _l = self
            .request_with_reply(
                "osal_nv_length",
                MessageType::SREQ,
                Subsystem::Sys,
                &[("id", ParameterValue::U16(100))],
                None,
            )
            .await;
        todo!()
    }
}

#[derive(Debug)]
pub enum NvMemoryAdapterError {
    IoError(std::io::Error),
    InvalidData,
    UnpiCommand(UnpiCommandError),
    Io(std::io::Error),
}

impl From<UnpiCommandError> for NvMemoryAdapterError {
    fn from(e: UnpiCommandError) -> Self {
        NvMemoryAdapterError::UnpiCommand(e)
    }
}

impl From<std::io::Error> for NvMemoryAdapterError {
    fn from(e: std::io::Error) -> Self {
        NvMemoryAdapterError::IoError(e)
    }
}
