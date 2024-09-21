//use pasts::Executor;
use futures::executor::block_on;
use rusty_zigbee_dongle::{
    coordinator::{Coordinator, CoordinatorError, LedStatus},
    zstack::cc253x::CC253X,
};

fn main() {
    #[cfg(feature = "log")]
    env_logger::init();

    let f = async {
        let cc2531 = CC253X::from_simple_serial("/dev/ttyACM2", 115_200)
            .await
            .unwrap();
        let version = cc2531.version().await.unwrap();
        println!("version: {:?}", version);
    };

    block_on(f);
}
