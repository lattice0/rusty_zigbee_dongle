use futures::executor::block_on;
use log::info;
use rusty_zigbee_dongle::{
    coordinator::{Coordinator, CoordinatorError, ZigbeeEvent},
    utils::sleep::sleep_forever,
    zstack::cc253x::CC253X,
};

fn main() {
    #[cfg(feature = "log")]
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let f = async {
        let mut cc2531 = CC253X::from_simple_serial("/dev/ttyACM2", 115_200)
            .await
            .unwrap();
        cc2531
            .set_on_event(Box::new(|event| {
                match event {
                    ZigbeeEvent::DeviceJoined {
                        network_address,
                        ieee_address,
                    } => {
                        info!("Device joined: {:?} {:?}", network_address, ieee_address);
                    }
                    ZigbeeEvent::DeviceAnnounce {
                        network_address,
                        ieee_address,
                    } => {
                        info!("Device announce: {:?} {:?}", network_address, ieee_address);
                    }
                    ZigbeeEvent::NetworkAddress {
                        network_address,
                        ieee_address,
                    } => {
                        info!("Network address: {:?} {:?}", network_address, ieee_address);
                    }
                    ZigbeeEvent::DeviceLeave(d) => {
                        info!("Device leave: {:?}", d);
                    }
                }
                #[allow(unreachable_code)]
                Ok(())
            }))
            .await
            .unwrap();

        let b = async {
            info!("starting...");
            cc2531.start().await.unwrap();
            let version = cc2531.version().await.unwrap();
            info!("version: {:?}", version);
            let device_info = cc2531.device_info().await.unwrap();
            info!("device_info: {:?}", device_info);
            cc2531.begin_startup().await.unwrap();
            let device_info = cc2531.device_info().await.unwrap();
            info!("device_info: {:?}", device_info);
            cc2531
                .permit_join(std::time::Duration::from_secs(100), None)
                .await
                .unwrap();
            info!("sleeping forever");
            sleep_forever().await.unwrap();
            Ok::<(), CoordinatorError>(())
        };
        futures::try_join!(b)
    };

    block_on(f).unwrap();
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use rusty_zigbee_dongle::{
        cc253x::CC253X,
        coordinator::{Coordinator, ResetType},
    };
    use std::path::PathBuf;

    #[ignore]
    #[test]
    fn hard_reset() {
        #[cfg(feature = "log")]
        env_logger::builder()
            .filter_level(log::LevelFilter::Trace)
            .init();

        let path = PathBuf::from("/dev/ttyACM1");
        block_on(async {
            let cc2531 = CC253X::from_path(path, 115_200).unwrap();
            cc2531.reset(ResetType::Hard).await.unwrap();
        });
    }
}
