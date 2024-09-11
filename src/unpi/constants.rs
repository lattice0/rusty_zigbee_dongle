use super::{commands::MAX_COMMAND_SIZE, parameters::ParameterValue};
use crate::utils::map::StaticMap;

pub const BEACON_MAX_DEPTH: u8 = 0x0f;
pub const DEF_NWK_RADIUS: u8 = 2 * BEACON_MAX_DEPTH;

pub mod af {
    pub enum InterpanCtl {
        CTL = 0,
        SET = 1,
        REG = 2,
        CHK = 3,
    }

    pub enum NetworkLatencyReq {
        NoLatencyReqs = 0,
        FastBeacons = 1,
        SlowBeacons = 2,
    }

    pub enum Otions {
        Preprocess = 4,
        LimitConcentrator = 8,
        AckRequest = 16,
        DiscvRoute = 32,
        EnSecurity = 64,
        SkipRouting = 128,
    }

    pub const DEFAULT_RADIUS: u8 = super::DEF_NWK_RADIUS;
}


#[derive(Debug, PartialEq, Clone)]
pub enum CommandStatus {
    Success,
    Failure,
    InvalidParam,
    NvItemInitialized,
    NvOperFailed,
    NvBadItemLen,
    MemError,
    BufferFull,
    UnsupportedMode,
    MacMemError,
    MacUnsupportedNotSpoort,
    MacBadState,
    MacNoResources,
    MacAckPending,
    MacNoTime,
    MacTxAborted,
    SapiInProgress,
    SapiTimeout,
    SapiInit,
    NotAuthorized,
    MalformedCmd,
    UnsupClusterCmd,
    OtaAbort,
    OtaImageInvalid,
    OtaWaitForData,
    OtaNoImageAvailable,
    OtaRequireMoreImage,
    ApsFail,
    ApsTableFull,
    ApsIllegalRequest,
    ApsInvalidBinding,
    ApsUnsupportedAttrib,
    ApsNotSupported,
    ApsNoAck,
    ApsDuplicateEntry,
    ApsNoBoundDevice,
    ApsNotAllowed,
    ApsNotAuthenticated,
    SecNoKey,
    SecOldFrmCount,
    SecMaxFrmCount,
    SecCcmFail,
    NwkInvalidParam,
    NwkInvalidRequest,
    NwkNotPermitted,
    NwkStartupFailure,
    NwkAlreadyPresent,
    NwkSyncFailure,
    NwkTableFull,
    NwkUnknownDevice,
    NwkUnsupportedAttribute,
    NwkNoNetworks,
    NwkLeaveUnconfirmed,
    NwkNoAck,
    NwkNoRoute,
    MacBeaconLoss,
    MacChannelAccessFailure,
    MacDenied,
    MacDisableTrxFailure,
    MacFailedSecurityCheck,
    MacFrameTooLong,
    MacInvalidGts,
    MacInvalidHandle,
    MacInvalidParameter,
    MacNoAck,
    MacNoBeacon,
    MacNoData,
    MacNoShortAddr,
    MacOutOfCap,
    MacPanidConflict,
    MacRealignment,
    MacTransactionExpired,
    MacTransactionOverflow,
    MacTxActive,
    MacUnavailableKey,
    MacUnsupportedAttribute,
    MacUnsupported,
    MacOnTimeTooLing,
    MacPastTime,
    MacTrackingOff,
    MacScanInProgress,
    MacSrcMatchInvalidIndex,
}

impl TryFrom<u8> for CommandStatus {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(CommandStatus::Success),
            0x01 => Ok(CommandStatus::Failure),
            0x02 => Ok(CommandStatus::InvalidParam),
            0x09 => Ok(CommandStatus::NvItemInitialized),
            0x0a => Ok(CommandStatus::NvOperFailed),
            0x0c => Ok(CommandStatus::NvBadItemLen),
            0x10 => Ok(CommandStatus::MemError),
            0x11 => Ok(CommandStatus::BufferFull),
            0x12 => Ok(CommandStatus::UnsupportedMode),
            0x13 => Ok(CommandStatus::MacMemError),
            0x18 => Ok(CommandStatus::MacUnsupportedNotSpoort),
            0x19 => Ok(CommandStatus::MacBadState),
            0x1a => Ok(CommandStatus::MacNoResources),
            0x1b => Ok(CommandStatus::MacAckPending),
            0x1c => Ok(CommandStatus::MacNoTime),
            0x1d => Ok(CommandStatus::MacTxAborted),
            0x20 => Ok(CommandStatus::SapiInProgress),
            0x21 => Ok(CommandStatus::SapiTimeout),
            0x22 => Ok(CommandStatus::SapiInit),
            0x7e => Ok(CommandStatus::NotAuthorized),
            0x80 => Ok(CommandStatus::MalformedCmd),
            0x81 => Ok(CommandStatus::UnsupClusterCmd),
            0x95 => Ok(CommandStatus::OtaAbort),
            0x96 => Ok(CommandStatus::OtaImageInvalid),
            0x97 => Ok(CommandStatus::OtaWaitForData),
            0x98 => Ok(CommandStatus::OtaNoImageAvailable),
            0x99 => Ok(CommandStatus::OtaRequireMoreImage),
            0xb1 => Ok(CommandStatus::ApsFail),
            0xb2 => Ok(CommandStatus::ApsTableFull),
            0xb3 => Ok(CommandStatus::ApsIllegalRequest),
            0xb4 => Ok(CommandStatus::ApsInvalidBinding),
            0xb5 => Ok(CommandStatus::ApsUnsupportedAttrib),
            0xb6 => Ok(CommandStatus::ApsNotSupported),
            0xb7 => Ok(CommandStatus::ApsNoAck),
            0xb8 => Ok(CommandStatus::ApsDuplicateEntry),
            0xb9 => Ok(CommandStatus::ApsNoBoundDevice),
            0xba => Ok(CommandStatus::ApsNotAllowed),
            0xbb => Ok(CommandStatus::ApsNotAuthenticated),
            0xa1 => Ok(CommandStatus::SecNoKey),
            0xa2 => Ok(CommandStatus::SecOldFrmCount),
            0xa3 => Ok(CommandStatus::SecMaxFrmCount),
            0xa4 => Ok(CommandStatus::SecCcmFail),
            0xc1 => Ok(CommandStatus::NwkInvalidParam),
            0xc2 => Ok(CommandStatus::NwkInvalidRequest),
            0xc3 => Ok(CommandStatus::NwkNotPermitted),
            0xc4 => Ok(CommandStatus::NwkStartupFailure),
            0xc5 => Ok(CommandStatus::NwkAlreadyPresent),
            0xc6 => Ok(CommandStatus::NwkSyncFailure),
            0xc7 => Ok(CommandStatus::NwkTableFull),
            0xc8 => Ok(CommandStatus::NwkUnknownDevice),
            0xc9 => Ok(CommandStatus::NwkUnsupportedAttribute),
            0xca => Ok(CommandStatus::NwkNoNetworks),
            0xcb => Ok(CommandStatus::NwkLeaveUnconfirmed),
            0xcc => Ok(CommandStatus::NwkNoAck),
            0xcd => Ok(CommandStatus::NwkNoRoute),
            0xe0 => Ok(CommandStatus::MacBeaconLoss),
            0xe1 => Ok(CommandStatus::MacChannelAccessFailure),
            0xe2 => Ok(CommandStatus::MacDenied),
            0xe3 => Ok(CommandStatus::MacDisableTrxFailure),
            0xe4 => Ok(CommandStatus::MacFailedSecurityCheck),
            0xe5 => Ok(CommandStatus::MacFrameTooLong),
            0xe6 => Ok(CommandStatus::MacInvalidGts),
            0xe7 => Ok(CommandStatus::MacInvalidHandle),
            0xe8 => Ok(CommandStatus::MacInvalidParameter),
            0xe9 => Ok(CommandStatus::MacNoAck),
            0xea => Ok(CommandStatus::MacNoBeacon),
            0xeb => Ok(CommandStatus::MacNoData),
            0xec => Ok(CommandStatus::MacNoShortAddr),
            0xed => Ok(CommandStatus::MacOutOfCap),
            0xee => Ok(CommandStatus::MacPanidConflict),
            0xef => Ok(CommandStatus::MacRealignment),
            0xf0 => Ok(CommandStatus::MacTransactionExpired),
            0xf1 => Ok(CommandStatus::MacTransactionOverflow),
            0xf2 => Ok(CommandStatus::MacTxActive),
            0xf3 => Ok(CommandStatus::MacUnavailableKey),
            0xf4 => Ok(CommandStatus::MacUnsupportedAttribute),
            0xf5 => Ok(CommandStatus::MacUnsupported),
            0xf6 => Ok(CommandStatus::MacOnTimeTooLing),
            0xf7 => Ok(CommandStatus::MacPastTime),
            0xf8 => Ok(CommandStatus::MacTrackingOff),
            0xfc => Ok(CommandStatus::MacScanInProgress),
            0xff => Ok(CommandStatus::MacSrcMatchInvalidIndex),
            _ => Err(()),
        }
    }
}

impl TryFrom<StaticMap<MAX_COMMAND_SIZE, &'static str, ParameterValue>> for CommandStatus {
    type Error = ();

    fn try_from(map: StaticMap<MAX_COMMAND_SIZE, &'static str, ParameterValue>) -> Result<Self, Self::Error> {
        let status = map
            .get(&"state")
            .ok_or(())?
            .try_into_u8()
            .map_err(|_| ())?;
        Ok(CommandStatus::try_from(status)?)
    }
}
