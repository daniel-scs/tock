//! A test of the USBC
//!
//! Creates a `SimpleClient` and sets it as the client
//! of the USB hardware interface.

use capsules::usb_simple::{SimpleClient};
use sam4l::usbc::{USBC};

static CLIENT: SimpleClient = SimpleClient::new(&USBC);

pub fn test() {
    unsafe {
        USBC.set_client(&CLIENT);

        CLIENT.enable();
        CLIENT.attach();
    }
}
