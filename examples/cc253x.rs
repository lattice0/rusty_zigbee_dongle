use rusty_zigbee_dongle::{
    cc253x::CC2531X,
    coordinator::{Coordinator, LedStatus},
};
use std::path::PathBuf;

fn main() {
    let mut cc2531 = CC2531X::from_path(PathBuf::from("/dev/ttyACM0"), 115_200).unwrap();
    // Not all firmware versions support LED write as far as I understood
    loop {
        cc2531.set_led(LedStatus::On).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(500));
        cc2531.set_led(LedStatus::Off).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}
