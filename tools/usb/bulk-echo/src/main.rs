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
use std::time::Duration;
use std::io::{stdin, stderr, Read, Write};

const VENDOR_ID: u16 = 0x6667;
const PRODUCT_ID: u16 = 0xabcd;

macro_rules! debug {
    [ $( $arg:expr ),+ ] => {{
        write!(stderr(), $( $arg ),+).expect("write");
        write!(stderr(), "\n").expect("write");
    }};
}

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
    // dh.reset().expect("Reset");
    dh.set_active_configuration(0)
        .expect("Setting active configuration");
    dh.claim_interface(0).expect("Claiming interface");

    // Unfortunately libusb doesn't provide an asynchronous interface,
    // so we'll make do here with blocking calls with short timeouts.
    // (Note that an async interface *is* available for the underlying
    // libusb C library.)

    loop {
        {
            // Get some input from stdin

            let mut buf = &mut [0; 3];
            let n = stdin().read(buf).expect("read");
            if n == 0 {
                // End of input
                break;
            }

            // Write it out to the device

            let endpoint = 2;
            let address = endpoint | 0 << 7; // OUT endpoint
            let timeout = Duration::from_secs(1);
            match dh.write_bulk(address, buf, timeout) {
                Ok(n) => debug!("Bulk wrote {} bytes", n),
                Err(Error::Timeout) => {
                    debug!("write timeout");
                    continue;
                }
                _ => panic!("write_bulk"),
            }
        }
        {
            // Read some data back from the device

            let endpoint = 1;
            let address = endpoint | 1 << 7; // IN endpoint
            let timeout = Duration::from_secs(1);
            let mut buf = &mut [0; 8];

            match dh.read_bulk(address, buf, timeout) {
                Ok(n) => debug!("Bulk read  {} bytes: {:?}", n, &buf[..n]),
                Err(Error::Timeout) => {
                    debug!("read timeout");
                    continue;
                }
                _ => panic!("read_bulk"),
            }
        }
    }

    println!("Done");
}
