//! Diagnostics for the USBC

extern crate kernel;
use kernel::hil;

use sam4l::usbc::{USBC};
use sam4l::usbc::data::*;

static mut EP0_BUF0: [u8; 8] = [99; 8];

const DEVICE_DESCRIPTOR: [u8; 18] =
    [ 18, // Length
       1, // DEVICE descriptor code
       2, // USB 2
       0, //      .0
       0, // Class
       0, // Subclass
       0, // Protocol
       8, // Max packet size
       0x66, 0x67,   // Vendor id
       0xab, 0xcd,   // Product id
       0x00, 0x01,   // Device release
       0, 0, 0,      // String indexes
       1  // Number of configurations
    ];
#[allow(non_upper_case_globals)]
static device_descriptor: &'static [u8] = &DEVICE_DESCRIPTOR;

struct Dummy { }

impl hil::usb::Client for Dummy {
    fn bus_reset(&self) {
        /* Ignore */
    }

    fn received_setup(&self, buf: &[u8]) {
        let s = hil::usb::SetupData::get(buf);
        match s {
            None => RequestResult::Error,
            Some(sd) => {
                match sd.standard_request_type() {
                    None => RequestResult::Error,
                    Some(r) => {
                        match r {
                            GetDescriptor{
                                descriptor_type: DescriptorType::Device,
                                descriptor_index: 0,
                                ..
                            } => RequestResult::Data(device_descriptor),
                            _ => RequestResult::Error,
                        }
                    }
                }
            }
        }
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
