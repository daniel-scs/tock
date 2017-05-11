//! A bare-bones client of the USB hardware interface
//!
//! It responds to standard device requests and can be enumerated.

use usb::*;
use kernel::hil::usb::*;

use core::cell::{RefCell};
use core::cmp::max;

pub struct<'a, C: Usbc> SimpleClient<'a> {
    controller: &'a C,
    state: RefCell<State>,
}

enum State {
    Init,
    CtrlIn{
        buf: &'static [u8],
        remaining_to_send: usize,
    }
}

impl<'a, C: Usbc>  SimpleClient<'a> {
    pub const fn new(controller: &'a C) -> Self {
        SimpleClient{
            controller: controller,
            state: RefCell::new(State::Init),
        }
    }
}

static EP0_BUF0: VolatileSlice<'static, u8> = VolatileSlice::new(&mut [99; 8]);

impl Client for SimpleClient {
    fn enable(&self) {
        self.controller.enable_device(false);
        self.controller.endpoint_set_buffer(0, EP0_BUF0);
        self.controller.endpoint_ctrl_out_enable(0);
    }

    fn attach(&self) {
        self.controller.attach();
    }

    fn bus_reset(&self) {
        /* Ignore */
    }

    fn ctrl_setup(&self) -> bool {
        let buf: &mut [u8] = [0: 8];
        copy_from_volatile_slice(buf, EP0_BUF0);

        SetupData::get(buf).map_or(InRequestResult::Error, |setup_data| {
            setup_data.get_standard_request().map_or(InRequestResult::Error, |request| {
                match request {
                    StandardDeviceRequest::GetDescriptor{
                        descriptor_type: DescriptorType::Device,
                        descriptor_index: 0,
                        ..
                    } => {
                        self.map_state(|state| {
                            *state = State::CtrlIn{
                                buf: device_descriptor,
                                remaining_to_send: device_descriptor.len(),
                            }
                        });
                        InRequestResult::Ok;
                    }
                    _ => InRequestResult::Error,
                }
            })
        })
    }

    fn ctrl_in(&self, packet_buf: &mut [u8]) -> CtrlInResult {
        self.map_state(|state| {
            match *state {
                State::CtrlIn{ buf, remaining_to_send } => {
                    if remaining_to_send > 0 {
                        let packet_bytes = max(packet_buf.size(), remaining_to_send);
                        let buf_start = buf.len() - remaining_to_send;
                        let buf_to_send = buf[buf_start .. buf_start + packet_bytes];
                        packet_buf.copy_from_slice(buf_to_send);

                        if let State::CtrlIn{ ref mut remaining_to_send } = *state {
                            *remaining_to_send -= packet_bytes;
                        }
                        CtrlInResult::Filled(packet_bytes)
                    }
                    else {
                        CtrlInResult::Error;
                    }
                }
                _ => CtrlInResult::Error;
            }
        }
    }

    fn received_setup_out(&self, buf: &[u8]) -> OutRequestResult {
        SetupData::get(buf).map_or(OutRequestResult::Error, |setup_data| {
            setup_data.get_standard_request().map_or(OutRequestResult::Error, |request| {
                match request {
                    StandardDeviceRequest::SetAddress{device_address} => {
                        OutRequestResult::Ok
                    }
                    => OutRequestResult::Error,
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
