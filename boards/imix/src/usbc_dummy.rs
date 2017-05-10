//! A test of the USBC
//!
//! Creates a `SimpleClient` and sets it as the client
//! of the USB hardware interface.

extern crate kernel;

use capsules::usb_simple::{SimpleClient};
use sam4l::usbc::{USBC};
use sam4l::usbc::data::*;

static CLIENT: SimpleClient = SimpleClient::new();

static mut EP0_BUF0: [u8; 8] = [99; 8];

// #[allow(unused_unsafe)]
pub fn test() {
    unsafe {
        USBC.set_client(&CLIENT);

        USBC.enable(Mode::device_at_speed(Speed::Low));

        let p0 = &EP0_BUF0 as *const u8 as *mut u8;
        USBC.endpoint_bank_set_buffer(EndpointIndex::new(0), BankIndex::Bank0, p0);

        let cfg0 = EndpointConfig::new(BankCount::Single,
                                       EndpointSize::Bytes8,
                                       EndpointDirection::Out,
                                       EndpointType::Control,
                                       EndpointIndex::new(0));
        USBC.endpoint_enable(0, cfg0);

        USBC.attach();

        // USBC.detach();

        // USBC.disable();
    }
}
