//use pasts::Executor;
use rusty_zigbee_dongle::{
    cc253x::CC253X,
    coordinator::{Coordinator, CoordinatorError, LedStatus},
};
use std::path::PathBuf;
use futures::executor::block_on;

fn main() {
    let looping = async {
        let cc2531 = CC253X::from_path(PathBuf::from("/dev/ttyACM0"), 115_200).unwrap();

        // Not all firmware versions support LED write as far as I understood
        let a = async {
            loop {
                cc2531.set_led(LedStatus::On).await.unwrap();
                std::thread::sleep(std::time::Duration::from_millis(500));
                cc2531.set_led(LedStatus::Off).await.unwrap();
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            //Ok::<(), CoordinatorError>(())
        };
        let b = async {
            println!("version: {:?}", cc2531.version().await);
            Ok::<(), CoordinatorError>(())
        };
        futures::join!(a,b)
    };
    
    let v = block_on(looping);
    println!("version: {:?}", v);
}
