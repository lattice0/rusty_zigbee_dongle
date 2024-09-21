use super::NvItemId;
use crate::zstack::unpi::serial::{request, request_with_reply};
use crate::zstack::unpi::subsystems::sys::{
    OsalNvLengthResponse, OsalNvReadRequest, OsalNvReadResponse,
};
use crate::zstack::unpi::LenTypeInfo;
use crate::{
    serial::SimpleSerial,
    subscription::SubscriptionService,
    zstack::unpi::{
        commands::{CommandRequest, CommandResponse},
        serial::UnpiCommandError,
        subsystems::sys::OsalNvLengthRequest,
        SUnpiPacket,
    },
};
use deku::{DekuContainerRead, DekuReader, DekuWriter};
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

    // helper proxy function
    pub async fn request<R: CommandRequest + DekuWriter>(
        &self,
        command: &R,
    ) -> Result<(), NvMemoryAdapterError> {
        let packet = SUnpiPacket::from_command_owned(LenTypeInfo::OneByte, command)?;
        Ok(request::<R, S>(&packet, self.serial.clone()).await?)
    }

    // // helper proxy function
    // async fn wait_for(
    //     &self,
    //     command_id: u8,
    //     message_type: MessageType,
    //     subsystem: Subsystem,
    //     timeout: Option<std::time::Duration>,
    // ) -> Result<SUnpiPacket, NvMemoryAdapterError> {
    //     Ok(wait_for(
    //         command_id,
    //         message_type,
    //         subsystem,
    //         self.subscriptions.clone(),
    //         timeout,
    //     )
    //     .await?)
    // }

    // helper proxy function
    async fn request_with_reply<
        R: CommandRequest + DekuWriter,
        Res: CommandResponse + for<'de> DekuReader<'de> + for<'de> DekuContainerRead<'de>,
    >(
        &self,
        command: &R,
        timeout: Option<std::time::Duration>,
    ) -> Result<Res, NvMemoryAdapterError> {
        Ok(request_with_reply::<R, S, Res>(
            &SUnpiPacket::from_command_owned(LenTypeInfo::OneByte, command)?,
            self.serial.clone(),
            self.subscriptions.clone(),
            timeout,
        )
        .await?)
    }

    pub async fn read_item<I: TryInto<NvItem>>(
        &self,
        id: NvItemId,
    ) -> Result<NvItem, NvMemoryAdapterError> {
        let r: OsalNvLengthResponse = self
            .request_with_reply(&OsalNvLengthRequest { id: id.into() }, None)
            .await?;
        let _len = r.length;
        let r: OsalNvReadResponse = self
            .request_with_reply(
                &OsalNvReadRequest {
                    id: id.into(),
                    offset: 0,
                },
                None,
            )
            .await?;
        println!("r: {:?}", r);
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
