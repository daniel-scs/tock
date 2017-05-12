//! A bare-bones client of the USB hardware interface
//!
//! It responds to standard device requests and can be enumerated.

use usb::*;
use kernel::common::volatile_slice::*;
use kernel::hil::usb::*;
use core::cell::{RefCell};
use core::cmp::max;

pub struct SimpleClient<'a, C: 'a> {
    controller: &'a C,
    state: RefCell<State>,
    ep0_buf: VolatileSlice<'a, u8>,
}

enum State {
    Init,
    CtrlIn{
        buf: &'static [u8]
    },
}

impl<'a, C: UsbController> SimpleClient<'a, C> {
    pub fn new(controller: &'a C) -> Self {
        let buf = static_init!(&'static [u8], &[0; 8]);
        let buf1 = unsafe { buf as &mut [u8] };
        SimpleClient{
            controller: controller,
            state: RefCell::new(State::Init),
            ep0_buf: VolatileSlice::new(buf1),
        }
    }

    fn map_state<F, R>(&self, f: F) -> R
        where F: FnOnce(&mut State) -> R
    {
        f(self.state.get_mut())
    }
}

impl<'a, C: UsbController> Client for SimpleClient<'a, C> {
    fn enable(&self) {
        self.controller.enable_device(false);
        self.controller.endpoint_set_buffer(0, self.ep0_buf);
        self.controller.endpoint_ctrl_out_enable(0);
    }

    fn attach(&self) {
        self.controller.attach();
    }

    fn bus_reset(&self) {
        /* Reconfigure */
    }

    fn ctrl_setup(&self) -> bool {
        let buf: &mut [u8] = &mut [0; 8];
        copy_from_volatile_slice(buf, self.ep0_buf);

        SetupData::get(buf).map_or(false, |setup_data| {
            setup_data.get_standard_request().map_or(false, |request| {
                match request {
                    StandardDeviceRequest::GetDescriptor{
                        descriptor_type: DescriptorType::Device,
                        descriptor_index: 0, ..  } => {

                        self.map_state(|state| {
                            *state = State::CtrlIn{ buf: DEVICE_DESCRIPTOR };
                        });
                        true
                    }
                    StandardDeviceRequest::SetAddress{device_address} => {
                        self.controller.set_address(device_address);
                        true
                    }
                    _ => false
                }
            })
        })
    }

    fn ctrl_in(&self) -> CtrlInResult {
        self.map_state(|state| {
            match *state {
                State::CtrlIn{ buf } => {
                    if buf.len() > 0 {
                        let packet_bytes = max(8, buf.len());
                        let packet = &buf[.. packet_bytes];
                        copy_volatile_from_slice(self.ep0_buf, packet);

                        let buf = &buf[packet_bytes ..];
                        *state = State::CtrlIn{ buf: buf };

                        CtrlInResult::Packet(packet_bytes, buf.len() == 0)
                    }
                    else {
                        CtrlInResult::Packet(0, true)
                    }
                }
                _ => CtrlInResult::Error
            }
        })
    }

    fn ctrl_out(&self /* , descriptor/bank */) {}

    fn ctrl_status(&self) {
        self.map_state(|state| {
            *state = State::Init
        })
    }

    fn ctrl_status_complete(&self) {}
}

static DEVICE_DESCRIPTOR: &'static [u8] =
   &[ 18, // Length
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
