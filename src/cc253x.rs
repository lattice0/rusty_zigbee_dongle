use crate::{
    coordinator::{Coordinator, CoordinatorError, LedStatus, ResetType},
    unpi::{
        commands::get_command_by_name, LenType, LenTypeInfo, MessageType, Subsystem, UnpiPacket,
    },
};
use serialport::SerialPort;
use std::{path::PathBuf, time::Duration};

const MAXIMUM_ZIGBEE_PAYLOAD_SIZE: usize = 255;

pub struct CC2531X {
    supports_led: Option<bool>,
    serial: Box<dyn SerialPort>,
}

impl CC2531X {
    pub fn from_path(path: PathBuf, baud_rate: u32) -> Result<Self, CoordinatorError> {
        let serial = serialport::new(path.to_str().unwrap(), baud_rate)
            .timeout(Duration::from_millis(10))
            .open()
            .map_err(|e| CoordinatorError::SerialOpen(e.to_string()))?;
        Ok(Self {
            serial,
            supports_led: None,
        })
    }
}

#[cfg(feature = "psila")]
impl Coordinator for CC2531X {
    type ZclFrame = psila_data::cluster_library::ClusterLibraryHeader;

    type ZclPayload<'a> = ([u8; MAXIMUM_ZIGBEE_PAYLOAD_SIZE], usize);

    type IeeAddress = ieee802154::mac::Address;

    fn start(&self) -> Result<(), CoordinatorError> {
        todo!()
    }

    fn stop(&self) -> Result<(), CoordinatorError> {
        todo!()
    }

    fn permit_join(
        &self,
        address: u16,
        duration: std::time::Duration,
    ) -> Result<(), CoordinatorError> {
        todo!()
    }

    fn reset(&self, reset_type: ResetType) -> Result<(), CoordinatorError> {
        todo!()
    }

    fn set_led(&self, led_status: LedStatus) -> Result<(), CoordinatorError> {
        // if (self.supports_led == None) {
        //     // Only zStack3x0 with 20210430 and greater support LED
        //     const zStack3x0 = this.version.product === ZnpVersion.zStack3x0;
        //     self.supports_led = !zStack3x0 || (zStack3x0 && parseInt(self.version.revision) >= 20210430);
        // }
        let command = get_command_by_name(&Subsystem::Util, "led_control")
            .ok_or(CoordinatorError::NoCommandWithName)?;
        let payload: &[u8] = todo!();
        let unpi_header = UnpiPacket::from_payload(
            (payload, LenTypeInfo::OneByte),
            (MessageType::SREQ, Subsystem::Util),
            command.id,
        );
        let buffer: &[u8] = todo!();
        self.serial
            .write_all(buffer)
            .map_err(|e| CoordinatorError::SerialWrite(e.to_string()))?;
    }

    fn request_network_address(addr: &str) -> Result<(), CoordinatorError> {
        todo!()
    }

    fn send_zcl_frame(
        &self,
        iee_addr: &Self::IeeAddress,
        network_address: u16,
        endpoint: u16,
        zcl_frame: &Self::ZclFrame,
        timeout: std::time::Duration,
        disable_response: bool,
        disable_recovery: bool,
        source_endpoint: Option<u32>,
    ) -> Result<Option<Self::ZclPayload<'static>>, CoordinatorError> {
        let payload: &[u8] = todo!();
        let unpi_header = UnpiPacket::from_payload(
            (payload, LenTypeInfo::OneByte),
            (MessageType::SREQ, Subsystem::Af),
            0x00,
        );
        let buffer: &[u8] = todo!();
        self.serial
            .write_all(buffer)
            .map_err(|e| CoordinatorError::SerialWrite(e.to_string()))?;
        Ok(None)
    }
}
/*
        const transactionID = this.nextTransactionID();
        const response = this.znp.waitFor(Type.AREQ, Subsystem.AF, 'dataConfirm', {transid: transactionID}, timeout);

        await this.znp.request(
            Subsystem.AF,
            'dataRequest',
            {
                dstaddr: destinationAddress,
                destendpoint: destinationEndpoint,
                srcendpoint: sourceEndpoint,
                clusterid: clusterID,
                transid: transactionID,
                options: 0,
                radius: radius,
                len: data.length,
                data: data,
            },
            response.ID,
        );

        let result = null;
        try {
            const dataConfirm = await response.start().promise;
            result = dataConfirm.payload.status;
        } catch {
            result = DataConfirmTimeout;
        }

        return result;
*/
