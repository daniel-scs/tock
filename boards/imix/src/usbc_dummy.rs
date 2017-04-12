//! Diagnostics for the USBC

extern crate kernel;
use kernel::hil;

use sam4l::usbc::{USBC};
use sam4l::usbc::data::*;

struct Dummy { }

impl hil::usb::Client for Dummy {
    fn received_setup(&self /* , descriptor/bank */) {}
    fn received_out(&self /* , descriptor/bank */) {}
}

static DUMMY: Dummy = Dummy {};

#[allow(unused_unsafe)]
pub unsafe fn test() {

    USBC.set_client(&DUMMY);

    let cfg0_buf: [u8; 8] = [0; 8];
    USBC.descriptors[0][0].set_addr(Buffer(&cfg0_buf as *const [u8] as *const u32 as u32));
    USBC.descriptors[0][0].set_packet_size(PacketSize::single(8));
    USBC.enable(Mode::Device(Speed::Low, None));

    let cfg0 = EndpointConfig::new(BankCount::Single,
                                   EndpointSize::Bytes8,
                                   EndpointDirection::Out,
                                   EndpointType::Control,
                                   EndpointIndex::new(0));
    USBC.endpoint_enable(0, cfg0);

    USBC.attach();

    // USBC.stimulate_interrupts();

    // USBC.detach();

    // USBC.disable();
}
