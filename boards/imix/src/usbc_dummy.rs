//! Diagnostics for the USBC

#![allow(unused_unsafe)]   // XXX

use sam4l::usbc::{USBC};
use sam4l::usbc::data::*;

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
