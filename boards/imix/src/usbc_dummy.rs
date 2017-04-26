//! Diagnostics for the USBC

extern crate kernel;
use kernel::hil;

use sam4l::usbc::{USBC};
use sam4l::usbc::data::*;

static CFG0_BUF0: [u8; 8] = [99; 8];
static CFG0_BUF1: [u8; 8] = [77; 8];

struct Dummy { }

impl hil::usb::Client for Dummy {
    fn received_setup(&self /* , descriptor/bank */) {}
    fn received_out(&self /* , descriptor/bank */) {}
}

static DUMMY: Dummy = Dummy {};

#[allow(unused_unsafe)]
pub unsafe fn test() {
    USBC.set_client(&DUMMY);

    USBC.enable(Mode::device_at_speed(Speed::Low));

    {
        USBC.descriptors[0][0].set_addr(Buffer(&CFG0_BUF0 as *const u8 as *mut u8));
        USBC.descriptors[0][0].set_packet_size(PacketSize::single(0));

        USBC.descriptors[0][1].set_addr(Buffer(&CFG0_BUF1 as *const u8 as *mut u8));
        USBC.descriptors[0][1].set_packet_size(PacketSize::single(0));

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
