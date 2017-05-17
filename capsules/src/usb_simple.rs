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

static DESCRIPTOR_STORAGE: &'static [u8] = &[0; 100];

static LANGUAGES: &'static [u16] = &[
    0x0409, // English (United States)
];

static MANUFACTURER_STRING: &'static str = "XYZ Corp.";

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
                                        let s = CopySlice::new(DESCRIPTOR_STORAGE);
                                        let len = DeviceDescriptor::default().write_to(s.as_mut());
                                        *state = State::CtrlIn{ buf: &DESCRIPTOR_STORAGE[ .. len] };
                                    });
                                    true
                                }
                                _ => false,
                            },
                            DescriptorType::Configuration => match descriptor_index {
                                0 => {
                                    // Place all the descriptors related to this configuration
                                    // into a buffer contiguously, starting with the last

                                    let mut storage_avail = DESCRIPTOR_STORAGE.len();

                                    let s = CopySlice::new(DESCRIPTOR_STORAGE);
                                    let d = InterfaceDescriptor::place(s.as_mut(),
                                                0,    // interface_number
                                                0,    // alternate_setting
                                                0,    // num_endpoints (exluding default control endpoint)
                                                0xff, // interface_class = vendor_specific
                                                0xab, // interface_subclass
                                                0,    // interface protocol
                                                0);   // string index
                                    storage_avail -= d.len();

                                    let s = CopySlice::new(&DESCRIPTOR_STORAGE[.. storage_avail]);
                                    let d = ConfigurationDescriptor::place(s.as_mut(),
                                               1,  // num interfaces
                                               0,  // config value
                                               0,  // string index
                                               ConfigurationAttributes::new(true, false),
                                               0,  // max power
                                               0); // length of related descriptors
                                    storage_avail -= d.len();

                                    self.map_state(|state| {
                                        *state = State::CtrlIn{ buf: &DESCRIPTOR_STORAGE[storage_avail ..] };
                                    });
                                    true
                                }
                                _ => false,
                            },
                            DescriptorType::String => {
                                if let Some(buf) = match descriptor_index {
                                                       0 => {
                                                            let mut storage_avail = DESCRIPTOR_STORAGE.len();
                                                            let s = CopySlice::new(DESCRIPTOR_STORAGE);
                                                            let d = LanguagesDescriptor::place(s.as_mut(), LANGUAGES);
                                                            storage_avail -= d.len();
                                                            Some(&DESCRIPTOR_STORAGE[storage_avail ..])
                                                       }
                                                       1 => if lang_id == LANGUAGES[0] {
                                                                let mut storage_avail = DESCRIPTOR_STORAGE.len();
                                                                let s = CopySlice::new(DESCRIPTOR_STORAGE);
                                                                let d = StringDescriptor::place(s.as_mut(), MANUFACTURER_STRING);
                                                                storage_avail -= d.len();
                                                                Some(&DESCRIPTOR_STORAGE[storage_avail ..])
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
                        } // match descriptor_type
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
