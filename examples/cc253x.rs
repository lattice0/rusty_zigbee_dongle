use pasts::Executor;
use rusty_zigbee_dongle::{
    cc253x::CC2531X,
    coordinator::{Coordinator, LedStatus},
};
use std::path::PathBuf;

fn main() {
    let looping = async {
        let cc2531 = CC2531X::from_path(PathBuf::from("/dev/ttyACM0"), 115_200).unwrap();
        // Not all firmware versions support LED write as far as I understood
        let a = async {
            loop {
                cc2531.set_led(LedStatus::On).await.unwrap();
                std::thread::sleep(std::time::Duration::from_millis(500));
                cc2531.set_led(LedStatus::Off).await.unwrap();
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        };
        let b = async {
            println!("version: {:?}", cc2531.version().await);
        };
        futures::join!(a, b);
    };
    let executor = Executor::default();

    // Calling `block_on()` starting executing queued tasks.
    executor.clone().block_on(async move {
        // Spawn tasks (without being queued)
        executor.spawn_boxed(looping);
    })
}
