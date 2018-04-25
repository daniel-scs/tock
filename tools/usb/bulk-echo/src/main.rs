//! This utility performs a simple test of usb functionality in Tock:
//! It reads from stdin and writes that data to a Bulk USB endpoint on
//! a connected device; in a separate thread, it reads that data back
//! and sends it to stdout.
//!
//! This utility depends on the `libusb` crate, which in turn requires
//! that the cross-platform (Windows, OSX, Linux) library
//! [libusb](http://libusb.info/) is installed on the host machine.
//!
//! To run the test, load the app in `examples/tests/usb` onto a device
//! running Tock; this app will enable the device's USB controller and
//! instruct it to respond to requests.
//!
//! Then, connect the device to a host machine's USB port and run this
//! program with something on stdin.

extern crate libusb;

use libusb::*;
// use std::thread::sleep;
use std::time::Duration;
use std::io::prelude::*;
use std::io::{stdin};

const VENDOR_ID: u16 = 0x6667;
const PRODUCT_ID: u16 = 0xabcd;

fn main() {
    let context = Context::new().expect("Creating context");

    let device_list = context.devices().expect("Getting device list");
    let mut dev = None;
    for d in device_list.iter() {
        let descr = d.device_descriptor().expect("Getting device descriptor");
        let matches = descr.vendor_id() == VENDOR_ID && descr.product_id() == PRODUCT_ID;
        if matches {
            dev = Some(d);
        }
    }

    let mut dh = dev.expect("Matching device not found")
        .open()
        .expect("Opening device");

    dh.set_active_configuration(0)
        .expect("Setting active configuration");

    dh.claim_interface(0).expect("Claiming interface");

    let mut buf = [0; 8];

    loop {
        let mut input = String::new();
        stdin().read_line(&mut input).expect("read stdin");

        {
            let endpoint = 2;
            let address = endpoint | 0 << 7; // OUT endpoint
            let timeout = Duration::from_secs(3);
            let n = dh.write_bulk(address, input.as_ref(), timeout).expect("write_bulk");
            println!("Bulk wrote {} bytes: {:?}", n, &buf[..n]);
        }
        {
            let endpoint = 1;
            let address = endpoint | 1 << 7; // IN endpoint
            let mut buf = &mut [0; 8];
            let timeout = Duration::from_secs(3);

            let n = dh.read_bulk(address, buf, timeout).expect("read_bulk");
            println!("Bulk read  {} bytes: {:?}", n, &buf[..n]);
        }
    }
}
