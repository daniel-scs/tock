//! A bare-bones client of the USB hardware interface
//!
//! It responds to standard device requests and can be enumerated.

use usb::*;
use kernel::common::volatile_cell::*;
use kernel::hil::usb::*;
use core::cell::Cell;
use core::cmp::min;

static LANGUAGES: &'static [u16] = &[
    0x0409, // English (United States)
];

static STRINGS: &'static [&'static str] = &[
    "XYZ Corp.",      // Manufacturer
    "The Zorpinator", // Product
    "Serial No. 5",   // Serial number
];

pub struct SimpleClient<'a, C: 'a> {
    controller: &'a C,
    state: Cell<State<'a>>,
    ep0_buf: &'a [VolatileCell<u8>],
    descriptor_buf: &'a [Cell<u8>],
}

#[derive(Copy, Clone)]
enum State<'a> {
    Init,
    CtrlIn{
        buf: &'a [Cell<u8>]
    },
    SetAddress,
}

pub const EP0_BUFLEN: usize = 8;
pub const DESCRIPTOR_BUFLEN: usize = 30;

impl<'a, C: UsbController> SimpleClient<'a, C> {
    pub fn new(controller: &'a C,
               ep0_buf: &'a [VolatileCell<u8>; EP0_BUFLEN],
               descriptor_buf: &'a [Cell<u8>; DESCRIPTOR_BUFLEN]) -> Self {

        SimpleClient{
            controller: controller,
            state: Cell::new(State::Init),
            ep0_buf: ep0_buf,
            descriptor_buf: descriptor_buf,
        }
    }
}

impl<'a, C: UsbController> Client for SimpleClient<'a, C> {
    fn enable(&self) {
        self.controller.endpoint_set_buffer(0, self.ep0_buf);
        self.controller.enable_device(false);
        self.controller.endpoint_ctrl_out_enable(0);
    }

    fn attach(&self) {
        self.controller.attach();
    }

    fn bus_reset(&self) {
        // Should the client initiate reconfiguration here?
        // For now, the hardware layer does it.
    }

    /// Handle a Control Setup transaction
    fn ctrl_setup(&self) -> CtrlSetupResult {
        SetupData::get(self.ep0_buf).map_or(CtrlSetupResult::ErrNoParse, |setup_data| {
            setup_data.get_standard_request().map_or_else(
                || { CtrlSetupResult::ErrNonstandardRequest },
                |request| {
                match request {
                    StandardDeviceRequest::GetDescriptor{ descriptor_type,
                                                          descriptor_index,
                                                          lang_id,
                                                          requested_length, } => {
                        match descriptor_type {
                            DescriptorType::Device => match descriptor_index {
                                0 => {
                                    let d = DeviceDescriptor {
                                                manufacturer_string: 1,
                                                product_string: 2,
                                                serial_number_string: 3,
                                                .. Default::default()
                                            };
                                    let len = d.write_to(self.descriptor_buf);
                                    let end = min(len, requested_length as usize);
                                    self.state.set(
                                        State::CtrlIn{
                                            buf: &self.descriptor_buf[ .. end]
                                        });
                                    CtrlSetupResult::Ok
                                }
                                _ => CtrlSetupResult::ErrInvalidDeviceIndex,
                            },
                            DescriptorType::Configuration => match descriptor_index {
                                0 => {
                                    // Place all the descriptors related to this configuration
                                    // into a buffer contiguously, starting with the last

                                    let mut storage_avail = self.descriptor_buf.len();
                                    let s = self.descriptor_buf;

                                    let di = InterfaceDescriptor::default();
                                    storage_avail -= di.write_to(&s[storage_avail - di.size() ..]);

                                    let dc = ConfigurationDescriptor {
                                                 num_interfaces: 1,
                                                 related_descriptor_length: di.size(),
                                                 .. Default::default() };
                                    storage_avail -= dc.write_to(&s[storage_avail - dc.size() ..]);

                                    let request_start = storage_avail;
                                    let request_end = min(request_start + (requested_length as usize),
                                                          self.descriptor_buf.len());
                                    self.state.set(
                                        State::CtrlIn{
                                            buf: &self.descriptor_buf[request_start .. request_end]
                                        });
                                    CtrlSetupResult::Ok
                                }
                                _ => CtrlSetupResult::ErrInvalidConfigurationIndex,
                            },
                            DescriptorType::String => {
                                if let Some(buf) = match descriptor_index {
                                       0 => {
                                            let d = LanguagesDescriptor{ langs: LANGUAGES };
                                            let len = d.write_to(self.descriptor_buf);
                                            let end = min(len, requested_length as usize);
                                            Some(&self.descriptor_buf[ .. end])
                                       }
                                       i if i > 0 && (i as usize) <= STRINGS.len() && lang_id == LANGUAGES[0] => {
                                            let d = StringDescriptor{ string: STRINGS[i as usize - 1] };
                                            let len = d.write_to(self.descriptor_buf);
                                            let end = min(len, requested_length as usize);
                                            Some(&self.descriptor_buf[ .. end])
                                       },
                                       _ => None,
                                   }
                                {
                                    self.state.set(State::CtrlIn{ buf: buf });
                                    CtrlSetupResult::Ok
                                }
                                else {
                                    CtrlSetupResult::ErrInvalidStringIndex
                                }
                            }
                            DescriptorType::DeviceQualifier => {
                                // We are full-speed only, so we must respond with a request error
                                CtrlSetupResult::ErrNoDeviceQualifier
                            }
                            _ => CtrlSetupResult::ErrUnrecognizedDescriptorType
                        } // match descriptor_type
                    }
                    StandardDeviceRequest::SetAddress{device_address} => {
                        // Load the address we've been assigned ...
                        self.controller.set_address(device_address);

                        // ... and when this request gets to the Status stage
                        // we will actually enable the address.
                        self.state.set(State::SetAddress);
                        CtrlSetupResult::Ok
                    }
                    StandardDeviceRequest::SetConfiguration{ .. } => {
                        // We have been assigned a particular configuration: fine!
                        CtrlSetupResult::Ok
                    }
                    _ => CtrlSetupResult::ErrUnrecognizedRequestType
                }
            })
        })
    }

    /// Handle a Control In transaction
    fn ctrl_in(&self) -> CtrlInResult {
        match self.state.get() {
            State::CtrlIn{ buf } => {
                if buf.len() > 0 {
                    let packet_bytes = min(8, buf.len());
                    let packet = &buf[.. packet_bytes];

                    // Copy a packet into the endpoint buffer
                    for (i, b) in packet.iter().enumerate() {
                        self.ep0_buf[i].set(b.get());
                    }

                    let buf = &buf[packet_bytes ..];
                    let transfer_complete = buf.len() == 0;

                    self.state.set(State::CtrlIn{ buf: buf });

                    CtrlInResult::Packet(packet_bytes, transfer_complete)
                }
                else {
                    CtrlInResult::Packet(0, true)
                }
            }
            _ => CtrlInResult::Error
        }
    }

    /// Handle a Control Out transaction
    ///   (for now, return an error)
    fn ctrl_out(&self, _packet_bytes: u32) -> CtrlOutResult {
        CtrlOutResult::Halted
    }

    fn ctrl_status(&self) {
        // Entered Status stage
    }

    /// Handle the completion of a Control transfer
    fn ctrl_status_complete(&self) {
        // Control Read: IN request acknowledged
        // Control Write: status sent

        match self.state.get() {
            State::SetAddress => {
                self.controller.enable_address();
            },
            _ => {}
        };
        self.state.set(State::Init);
    }
}
