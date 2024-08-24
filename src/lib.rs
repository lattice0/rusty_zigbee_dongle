pub enum CommandType{
    Poll,
    SREQ,
    AREQ,
    SRSP,
    RES0,
    RES1,
    RES2,
    RES3
}

impl TryFrom<u8> for CommandType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CommandType::Poll),
            1 => Ok(CommandType::SREQ),
            2 => Ok(CommandType::AREQ),
            3 => Ok(CommandType::SRSP),
            4 => Ok(CommandType::RES0),
            5 => Ok(CommandType::RES1),
            6 => Ok(CommandType::RES2),
            7 => Ok(CommandType::RES3),
            _ => Err(()),
        }
    }
}

impl Into<u8> for CommandType {
    fn into(self) -> u8 {
        match self {
            CommandType::Poll => 0,
            CommandType::SREQ => 1,
            CommandType::AREQ => 2,
            CommandType::SRSP => 3,
            CommandType::RES0 => 4,
            CommandType::RES1 => 5,
            CommandType::RES2 => 6,
            CommandType::RES3 => 7,
        }
    }
}

pub enum Subsystem {
    RpcSysRes0,
    RpcSysSys,
    RpcSysMac,
    RpcSysNwk,
    RpcSysAf,
    RpcSysZdo,
    RpcSysSapi,
    RpcSysUtil,
    RpcSysDbg,
    RpcSysApp,
    RpcSysRcaf,
    RpcSysRcn,
    RpcSysRcnClient,
    RpcSysBoot,
    RpcSysZiptest,
    RpcSysDebug,
    RpcSysPeripherals,
    RpcSysNfc,
    RpcSysPbNwkMgr,
    RpcSysPbGw,
    RpcSysPbOtaMgr,
    RpcSysBleSpnp,
    RpcSysBleHci,
    RpcSysResv01,
    RpcSysResv02,
    RpcSysResv03,
    RpcSysResv04,
    RpcSysResv05,
    RpcSysResv06,
    RpcSysResv07,
    RpcSysResv08,
    RpcSysSrvCtr,
}

impl TryFrom<u8> for Subsystem {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Subsystem::RpcSysRes0),
            1 => Ok(Subsystem::RpcSysSys),
            2 => Ok(Subsystem::RpcSysMac),
            3 => Ok(Subsystem::RpcSysNwk),
            4 => Ok(Subsystem::RpcSysAf),
            5 => Ok(Subsystem::RpcSysZdo),
            6 => Ok(Subsystem::RpcSysSapi),
            7 => Ok(Subsystem::RpcSysUtil),
            8 => Ok(Subsystem::RpcSysDbg),
            9 => Ok(Subsystem::RpcSysApp),
            10 => Ok(Subsystem::RpcSysRcaf),
            11 => Ok(Subsystem::RpcSysRcn),
            12 => Ok(Subsystem::RpcSysRcnClient),
            13 => Ok(Subsystem::RpcSysBoot),
            14 => Ok(Subsystem::RpcSysZiptest),
            15 => Ok(Subsystem::RpcSysDebug),
            16 => Ok(Subsystem::RpcSysPeripherals),
            17 => Ok(Subsystem::RpcSysNfc),
            18 => Ok(Subsystem::RpcSysPbNwkMgr),
            19 => Ok(Subsystem::RpcSysPbGw),
            20 => Ok(Subsystem::RpcSysPbOtaMgr),
            21 => Ok(Subsystem::RpcSysBleSpnp),
            22 => Ok(Subsystem::RpcSysBleHci),
            23 => Ok(Subsystem::RpcSysResv01),
            24 => Ok(Subsystem::RpcSysResv02),
            25 => Ok(Subsystem::RpcSysResv03),
            26 => Ok(Subsystem::RpcSysResv04),
            27 => Ok(Subsystem::RpcSysResv05),
            28 => Ok(Subsystem::RpcSysResv06),
            29 => Ok(Subsystem::RpcSysResv07),
            30 => Ok(Subsystem::RpcSysResv08),
            31 => Ok(Subsystem::RpcSysSrvCtr),
            _ => Err(()),
        }
    }
}


// var subSys = {
//     "RPC_SYS_RES0": 0,
//     "RPC_SYS_SYS": 1,
//     "RPC_SYS_MAC": 2,
//     "RPC_SYS_NWK": 3,
//     "RPC_SYS_AF": 4,
//     "RPC_SYS_ZDO": 5,
//     "RPC_SYS_SAPI": 6,
//     "RPC_SYS_UTIL": 7,
//     "RPC_SYS_DBG": 8,
//     "RPC_SYS_APP": 9,
//     "RPC_SYS_RCAF": 10,
//     "RPC_SYS_RCN": 11,
//     "RPC_SYS_RCN_CLIENT": 12,
//     "RPC_SYS_BOOT": 13,
//     "RPC_SYS_ZIPTEST": 14,
//     "RPC_SYS_DEBUG": 15,
//     "RPC_SYS_PERIPHERALS": 16,
//     "RPC_SYS_NFC": 17,
//     "RPC_SYS_PB_NWK_MGR": 18,
//     "RPC_SYS_PB_GW": 19,
//     "RPC_SYS_PB_OTA_MGR": 20,
//     "RPC_SYS_BLE_SPNP": 21,
//     "RPC_SYS_BLE_HCI": 22,
//     "RPC_SYS_RESV01": 23,
//     "RPC_SYS_RESV02": 24,
//     "RPC_SYS_RESV03": 25,
//     "RPC_SYS_RESV04": 26,
//     "RPC_SYS_RESV05": 27,
//     "RPC_SYS_RESV06": 28,
//     "RPC_SYS_RESV07": 29,
//     "RPC_SYS_RESV08": 30,
//     "RPC_SYS_SRV_CTR": 31
// };

#[cfg(test)]
mod tests {
    #[test]
    fn t() {
        use serialport::{available_ports, SerialPortType};

        match available_ports() {
            Ok(ports) => {
                match ports.len() {
                    0 => println!("No ports found."),
                    1 => println!("Found 1 port:"),
                    n => println!("Found {} ports:", n),
                };
                for p in ports {
                    println!("  {}", p.port_name);
                    match p.port_type {
                        SerialPortType::UsbPort(info) => {
                            println!("    Type: USB");
                            println!("    VID:{:04x} PID:{:04x}", info.vid, info.pid);
                            println!(
                                "     Serial Number: {}",
                                info.serial_number.as_ref().map_or("", String::as_str)
                            );
                            println!(
                                "      Manufacturer: {}",
                                info.manufacturer.as_ref().map_or("", String::as_str)
                            );
                            println!(
                                "           Product: {}",
                                info.product.as_ref().map_or("", String::as_str)
                            );
                            #[cfg(feature = "usbportinfo-interface")]
                            println!(
                                "         Interface: {}",
                                info.interface
                                    .as_ref()
                                    .map_or("".to_string(), |x| format!("{:02x}", *x))
                            );
                        }
                        SerialPortType::BluetoothPort => {
                            println!("    Type: Bluetooth");
                        }
                        SerialPortType::PciPort => {
                            println!("    Type: PCI");
                        }
                        SerialPortType::Unknown => {
                            println!("    Type: Unknown");
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("{:?}", e);
                eprintln!("Error listing serial ports");
            }
        }
    }
}
