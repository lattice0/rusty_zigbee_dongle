use crate::{
    parameters::ParameterValue,
    serial::SimpleSerial,
    subscription::SubscriptionService,
    zstack::unpi::{
        commands::ParametersValueMap,
        serial::{request_with_reply, UnpiCommandError},
        SUnpiPacket, Subsystem,
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

    // // helper proxy function
    // async fn wait_for(
    //     &self,
    //     name: &str,
    //     message_type: MessageType,
    //     subsystem: Subsystem,
    //     timeout: Option<std::time::Duration>,
    // ) -> Result<(SUnpiPacket, Command), NvMemoryAdapterError> {
    //     Ok(wait_for(
    //         name,
    //         message_type,
    //         subsystem,
    //         self.subscriptions.clone(),
    //         timeout,
    //     )
    //     .await?)
    // }

    // // helper proxy function
    // async fn request(
    //     &self,
    //     name: &str,
    //     subsystem: Subsystem,
    //     parameters: &[(&'static str, ParameterValue)],
    //     _timeout: Option<std::time::Duration>,
    // ) -> Result<(), NvMemoryAdapterError> {
    //     request(name, subsystem, parameters, self.serial.clone()).await?;
    //     Ok(())
    // }

    // helper proxy function
    async fn request_with_reply(
        &self,
        name: &str,
        subsystem: Subsystem,
        parameters: &[(&'static str, ParameterValue)],
        timeout: Option<std::time::Duration>,
    ) -> Result<ParametersValueMap, NvMemoryAdapterError> {
        Ok(request_with_reply(
            name,
            subsystem,
            parameters,
            self.serial.clone(),
            self.subscriptions.clone(),
            timeout,
        )
        .await?)
    }

    pub async fn read_item<I: TryInto<NvItem>>(
        &self,
        id: u16,
    ) -> Result<NvItem, NvMemoryAdapterError> {
        let r = self
            .request_with_reply(
                "osal_nv_length",
                Subsystem::Sys,
                &[("id", ParameterValue::U16(id))],
                None,
            )
            .await?;
        println!("r: {:?}", r);
        let status = r
            .get(&"status")
            .ok_or(NvMemoryAdapterError::MissingResponse)?;
        let len = r
            .get(&"status")
            .ok_or(NvMemoryAdapterError::MissingResponse)?;
        let value = r
            .get(&"value")
            .ok_or(NvMemoryAdapterError::MissingResponse)?;
        println!("{:?}, {:?}, {:?}", status, len, value);
        todo!()
    }
}

pub enum NvItem {}

impl TryFrom<()> for NvItem {
    type Error = NvMemoryAdapterError;

    fn try_from(_: ()) -> Result<Self, Self::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub enum NvMemoryAdapterError {
    IoError(std::io::Error),
    InvalidData,
    UnpiCommand(UnpiCommandError),
    Io(std::io::Error),
    MissingResponse,
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
