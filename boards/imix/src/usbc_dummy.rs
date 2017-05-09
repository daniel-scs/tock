//! Diagnostics for the USBC

extern crate kernel;
use kernel::hil;

use sam4l::usbc::{USBC};
use sam4l::usbc::data::*;

static mut EP0_BUF0: [u8; 8] = [99; 8];

struct Dummy { }

impl hil::usb::Client for Dummy {
    fn bus_reset(&self) {
        /* Ignore */
    }

    fn received_setup(&self) {
    }

    fn received_out(&self /* , descriptor/bank */) {}
}

static DUMMY: Dummy = Dummy {};

// #[allow(unused_unsafe)]
pub fn test() {
    unsafe {
        USBC.set_client(&DUMMY);

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
