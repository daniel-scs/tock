//! A bare-bones client of the USB hardware interface.
//!
//! It responds to standard device requests and can be enumerated.

use usb::*;
use kernel::hil::usb::*;

pub struct SimpleClient { }

impl SimpleClient {
    pub const fn new() -> Self {
        SimpleClient{}
    }
}

impl Client for SimpleClient {
    fn bus_reset(&self) {
        /* Ignore */
    }

    fn received_setup_in(&self, buf: &[u8]) -> InRequestResult {
        SetupData::get(buf).map_or(InRequestResult::Error, |setup_data| {
            setup_data.standard_request_type().map_or(InRequestResult::Error, |request| {
                match request {
                    StandardDeviceRequest::GetDescriptor{
                        descriptor_type: DescriptorType::Device,
                        descriptor_index: 0,
                        ..
                    } => InRequestResult::Data(device_descriptor),
                    _ => InRequestResult::Error,
                }
            })
        })
    }

    fn received_out(&self /* , descriptor/bank */) {
    }
}

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
