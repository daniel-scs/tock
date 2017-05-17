//! A bare-bones client of the USB hardware interface
//!
//! It responds to standard device requests and can be enumerated.

use usb::*;
use kernel::common::volatile_slice::*;
use kernel::common::copy_slice::*;
use kernel::hil::usb::*;
use core::cell::{RefCell};
use core::ops::DerefMut;
use core::cmp::max;

pub struct SimpleClient<'a, C: 'a> {
    controller: &'a C,
    state: RefCell<State>,
    ep0_buf: VolatileSlice<u8>,
    descr_storage: [u8; 100],
}

enum State {
    Init,
    CtrlIn{
        buf: &'static [u8]
    },
    SetAddress,
}

/// Storage for endpoint 0 packets
static EP0_BUF: &'static [u8] = &[0; 8];

impl<'a, C: UsbController> SimpleClient<'a, C> {
    pub fn new(controller: &'a C) -> Self {
        SimpleClient{
            controller: controller,
            state: RefCell::new(State::Init),
            ep0_buf: VolatileSlice::new(EP0_BUF),
        }
    }

    fn map_state<F, R>(&self, f: F) -> R
        where F: FnOnce(&mut State) -> R
    {
        let mut s = self.state.borrow_mut();
        f(s.deref_mut())
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
                    StandardDeviceRequest::GetDescriptor{ descriptor_type, descriptor_index, lang_id } => {
                        match descriptor_type {
                            DescriptorType::Device => match descriptor_index {
                                0 => {
                                    self.map_state(|state| {
                                        *state = State::CtrlIn{ buf: DEVICE_DESCRIPTOR };
                                    });
                                    true
                                }
                                _ => false,
                            },
                            DescriptorType::Configuration => match descriptor_index {
                                0 => {
                                    let s = CopySlice::new(DESCRIPTOR_STORAGE);
                                    let d = ConfigurationDescriptor::place(s.as_mut(),
                                               1, // num interfaces
                                               0, // config value
                                               0, // string index
                                               ConfigurationAttributes::new(true, false),
                                               0, // max power
                                               0  // length of related descriptors
                                               );
                                    self.map_state(|state| {
                                        *state = State::CtrlIn{ buf: DESCRIPTOR_STORAGE.as_ref() };
                                    });
                                    true
                                }
                                _ => false,
                            DescriptorType::String => {
                                if let Some(buf) = match descriptor_index {
                                                       0 => Some(LANG0_DESCRIPTOR),
                                                       1 => if lang_id == 0x0409 {
                                                                Some(MANUFACTURER_DESCRIPTOR)
                                                            }
                                                            else {
                                                                None
                                                            },
                                                       _ => None,
                                                   }
                                {
                                    self.map_state(|state| {
                                        *state = State::CtrlIn{ buf: buf };
                                    });
                                    true
                                }
                                else {
                                    false
                                }
                            }
                            _ => false,
                        }
                    }
                    StandardDeviceRequest::SetAddress{device_address} => {
                        // Load the address we've been assigned ...
                        self.controller.set_address(device_address);

                        // ... and when this request gets to the Status stage
                        // we will actually enable the address.
                        self.map_state(|state| {
                            *state = State::SetAddress;
                        });
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
                        self.ep0_buf.copy_from_slice(packet);

                        let buf = &buf[packet_bytes ..];
                        let transfer_complete = buf.len() == 0;

                        *state = State::CtrlIn{ buf: buf };

                        CtrlInResult::Packet(packet_bytes, transfer_complete)
                    }
                    else {
                        CtrlInResult::Packet(0, true)
                    }
                }
                _ => CtrlInResult::Error
            }
        })
    }

    fn ctrl_out(&self, _packet_bytes: u32) -> CtrlOutResult {
        CtrlOutResult::Halted
    }

    fn ctrl_status(&self) {
        self.map_state(|state| {
            match *state {
                State::SetAddress => {
                    self.controller.enable_address();
                },
                _ => {}
            };
            *state = State::Init
        })
    }

    fn ctrl_status_complete(&self) {
        // IN request acknowledged
    }
}

static DESCRIPTOR_STORAGE: &'static [u8] = &[0; 100];

static LANG0_DESCRIPTOR: &'static [u8] =
    &[ 4, // Length
       DescriptorType::String as u8, // STRING descriptor code
       0x09, 0x04 // English (United States)
     ];

static MANUFACTURER_DESCRIPTOR: &'static [u8] =
    &[ 5, // Length
       DescriptorType::String as u8, // STRING descriptor code
       b'X', b'Y', b'Z'
     ];

static DEVICE_DESCRIPTOR: &'static [u8] =
   &[ 18, // Length
       DescriptorType::Device as u8, // DEVICE descriptor code
       0, // USB 2
       2, //
       0, // Class
       0, // Subclass
       0, // Protocol
       8, // Max packet size
       0x66, 0x67,   // Vendor id
       0xab, 0xcd,   // Product id
       0x00, 0x01,   // Device release
       1, // Manufacturer string index
       0, // Product string index
       0, // Serial Number string index
       1  // Number of configurations
    ];
