use super::{commands::Command, Subsystem};
use sys::COMMANDS_SYS;
use util::COMMANDS_UTIL;
use zdo::COMMANDS_ZDO;

pub mod sys;
pub mod util;
pub mod zdo;

pub const SUBSYSTEMS: &[(Subsystem, &[Command])] = &[
    (Subsystem::Util, COMMANDS_UTIL),
    (Subsystem::Zdo, COMMANDS_ZDO),
    (Subsystem::Sys, COMMANDS_SYS),
];
