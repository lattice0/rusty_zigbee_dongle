//use pasts::Executor;
use futures::executor::block_on;
use rusty_zigbee_dongle::{
    cc253x::CC253X,
    coordinator::{Coordinator, CoordinatorError, LedStatus},
};
use std::path::PathBuf;

fn main() {
    let f = async {
        let cc2531 = CC253X::from_path(PathBuf::from("/dev/ttyACM0"), 115_200).unwrap();

        // Not all firmware versions support LED write as far as I understood
        let a = async {
            
            cc2531.set_led(LedStatus::On).await.unwrap();
            Ok::<(), CoordinatorError>(())
        };
        let b = async {
            println!("version: {:?}", cc2531.version().await);
            Ok::<(), CoordinatorError>(())
        };
        futures::try_join!(a, b)
    };

    block_on(f).unwrap();
}
