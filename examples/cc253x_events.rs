//use pasts::Executor;
use futures::executor::block_on;
use rusty_zigbee_dongle::{
    cc253x::CC253X,
    coordinator::{Coordinator, CoordinatorError, LedStatus, ZigbeeEvent},
};
use std::path::PathBuf;

fn main() {
    #[cfg(feature = "log")]
    env_logger::builder()
        .format_timestamp(None)
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
                    ZigbeeEvent::DeviceLeave(_) => todo!(),
                }
                Ok(())
            }))
            .await
            .unwrap();

        // Not all firmware versions support LED write as far as I understood
        let a = async {
            cc2531.set_led(LedStatus::On).await.unwrap();
            Ok::<(), CoordinatorError>(())
        };
        let b = async {
            let version = cc2531.version().await.unwrap();
            println!("version: {:?}", version);
            cc2531
                .permit_join(std::time::Duration::from_secs(10), None)
                .await
                .unwrap();
            Ok::<(), CoordinatorError>(())
        };
        futures::try_join!(a, b)
    };

    block_on(f).unwrap();
}
