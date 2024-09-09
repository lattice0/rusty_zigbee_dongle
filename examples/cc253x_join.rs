//use pasts::Executor;
use futures::executor::block_on;
use log::info;
use rusty_zigbee_dongle::{
    cc253x::CC253X,
    coordinator::{Coordinator, CoordinatorError, ZigbeeEvent},
    utils::sleep::sleep_forever,
};
use std::path::PathBuf;

fn main() {
    #[cfg(feature = "log")]
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let f = async {
        let mut cc2531 = CC253X::from_path(PathBuf::from("/dev/ttyACM0"), 115_200).unwrap();

        cc2531
            .set_on_event(Box::new(|event| {
                match event {
                    ZigbeeEvent::DeviceJoined {
                        network_address,
                        ieee_address,
                    } => {
                        println!("Device joined: {:?} {:?}", network_address, ieee_address);
                    }
                    ZigbeeEvent::DeviceAnnounce {
                        network_address,
                        ieee_address,
                    } => {
                        println!("Device announce: {:?} {:?}", network_address, ieee_address);
                    }
                    ZigbeeEvent::NetworkAddress {
                        network_address,
                        ieee_address,
                    } => {
                        println!("Network address: {:?} {:?}", network_address, ieee_address);
                    }
                    ZigbeeEvent::DeviceLeave(d) => {
                        println!("Device leave: {:?}", d);
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
