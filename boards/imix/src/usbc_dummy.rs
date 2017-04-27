//! Diagnostics for the USBC

extern crate kernel;
use kernel::hil;

use sam4l::usbc::{USBC};
use sam4l::usbc::data::*;

static EP0_BUF0: [u8; 8] = [99; 8];

struct Dummy { }

impl hil::usb::Client for Dummy {
    fn received_setup(&self /* , descriptor/bank */) {}
    fn received_out(&self /* , descriptor/bank */) {}
}

static DUMMY: Dummy = Dummy {};

#[allow(unused_unsafe)]
pub unsafe fn test() {
    println!("Buffer at {:?}", &EP0_BUF0 as *const u8);

    USBC.set_client(&DUMMY);

    USBC.enable(Mode::device_at_speed(Speed::Low));

    {
        USBC.endpoint_bank_set_buffer(EndpointIndex::new(0), BankIndex::Bank0,
                                      &EP0_BUF0);

        let cfg0 = EndpointConfig::new(BankCount::Single,
                                       EndpointSize::Bytes8,
                                       EndpointDirection::Out,
                                       EndpointType::Control,
                                       EndpointIndex::new(0));
        USBC.endpoint_enable(0, cfg0);
    }

    USBC.attach();

    // USBC.detach();

    // USBC.disable();
}
