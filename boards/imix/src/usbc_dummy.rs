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
