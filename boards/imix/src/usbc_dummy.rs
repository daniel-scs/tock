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

#[allow(unused_unsafe)]
pub unsafe fn test() {

    let cfg0_buf: [u8; 8] = [0; 8];
    USBC.descriptors[0][0].set_addr(Buffer::new(&cfg0_buf as *const [u8] as u32));
    USBC.descriptors[0][0].set_packet_size(PacketSize::single(8));

    let cfg0_in = EndpointConfig::new(BankCount::Single,
                                      EndpointSize::Bytes8,
                                      EndpointDirection::In,
                                      EndpointType::Control,
                                      0);
    USBC.enable_endpoint(0, cfg0_in);

    let cfg0_out = EndpointConfig::new(BankCount::Single,
                                       EndpointSize::Bytes8,
                                       EndpointDirection::In,
                                       EndpointType::Control,
                                       0);
    USBC.enable_endpoint(0, cfg0_out);

    println!("Mode: {:?}", USBC.state());

    let mode = Mode::Device(Speed::Low);
    USBC.enable(mode);
    println!("Mode: {:?}", USBC.state());

    USBC.attach();
    println!("Mode: {:?}", USBC.state());

    /*
    USBC.detach();
    println!("Mode: {:?}", USBC.state());
    */

    USBC.disable();
    println!("Mode: {:?}", USBC.state());
}
