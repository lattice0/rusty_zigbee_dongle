//use pasts::Executor;
use futures::executor::block_on;
use rusty_zigbee_dongle::{
    coordinator::{Coordinator, CoordinatorError, LedStatus},
    utils::sleep::async_delay,
    zstack::cc253x::CC253X,
};

#[allow(unreachable_code)]
fn main() {
    #[cfg(feature = "log")]
    env_logger::init();

    let f = async {
        let cc2531 = CC253X::from_simple_serial("/dev/ttyACM2", 115_200)
            .await
            .unwrap();

        // Not all firmware versions support LED write as far as I understood
        let a = async {
            loop {
                cc2531.set_led(LedStatus::On).await.unwrap();
                async_delay(std::time::Duration::from_secs(1))
                    .await
                    .unwrap();
                cc2531.set_led(LedStatus::Off).await.unwrap();
                async_delay(std::time::Duration::from_secs(1))
                    .await
                    .unwrap();
            }
            Ok::<(), CoordinatorError>(())
        };
        let b = async {
            let version = cc2531.version().await.unwrap();
            println!("version: {:?}", version);
            Ok::<(), CoordinatorError>(())
        };
        futures::try_join!(a, b)
    };

    block_on(f).unwrap();
}
