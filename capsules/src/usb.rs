//! Platform-independent USB 2.1 protocol library

use core::fmt;
use core::convert::From;

#[derive(Debug)]
#[repr(C, packed)]
pub struct SetupData {
    pub request_type: DeviceRequestType,
    pub request_code: u8,
    pub value: u16,
    pub index: u16,
    pub length: u16,
}

impl SetupData {
    /// Create a `SetupData` structure from a buffer received from the wire
    pub fn get(buf: &[u8]) -> Option<Self> {
        if buf.len() != 8 {
            return None;
        }
        let (rt, buf) = buf.split_at(1);
        let (rc, buf) = buf.split_at(1);
        let (vl, buf) = buf.split_at(2);
        let (ix, buf) = buf.split_at(2);
        let ln = buf;
        Some(SetupData {
            request_type: DeviceRequestType(rt[0]),
            request_code: rc[0],
            value: get_u16(vl).unwrap(),
            index: get_u16(ix).unwrap(),
            length: get_u16(ln).unwrap(),
        })
    }

    /// If the `SetupData` represents a standard device request, return it
    pub fn get_standard_request(&self) -> Option<StandardDeviceRequest> {
        match self.request_type.request_type() {
            RequestType::Standard =>
                match self.request_code {
                    0 => Some(StandardDeviceRequest::GetStatus{
                             recipient_index: self.index
                         }),
                    1 => Some(StandardDeviceRequest::ClearFeature{
                            feature: FeatureSelector::get(self.value),
                            recipient_index: self.index,
                         }),
                    3 => Some(StandardDeviceRequest::SetFeature{
                            feature: FeatureSelector::get(self.value),
                            test_mode: (self.index >> 8) as u8,
                            recipient_index: self.index & 0xff,
                         }),
                    5 => Some(StandardDeviceRequest::SetAddress{
                            device_address: self.value
                         }),
                    6 => {
                        get_descriptor_type((self.value >> 8) as u8).map_or(None, |dt| {
                            Some(StandardDeviceRequest::GetDescriptor{
                                    descriptor_type: dt,
                                    descriptor_index: (self.value & 0xff) as u8,
                                    lang_id: self.index,
                            })
                        })
                    }
                    7 => {
                        get_set_descriptor_type((self.value >> 8) as u8).map_or(None, |dt| {
                            Some(StandardDeviceRequest::SetDescriptor{
                                descriptor_type: dt,
                                descriptor_index: (self.value & 0xff) as u8,
                                lang_id: self.index,
                                descriptor_length: self.length
                            })
                        })
                    }
                    8 => Some(StandardDeviceRequest::GetConfiguration),
                    9 => Some(StandardDeviceRequest::SetConfiguration{
                            configuration: (self.value & 0xff) as u8
                         }),
                    10 => Some(StandardDeviceRequest::GetInterface{
                              interface: self.index
                          }),
                    11 => Some(StandardDeviceRequest::SetInterface),
                    12 => Some(StandardDeviceRequest::SynchFrame),
                    _ => None,
                },
            _ => None,
        }
    }
}

fn get_u16(buf: &[u8]) -> Option<u16> {
    if buf.len() != 2 {
        return None;
    }
    Some ((buf[0] as u16) | ((buf[1] as u16) << 8))
}

#[derive(Debug)]
pub enum StandardDeviceRequest {
    GetStatus{
        recipient_index: u16,
    },
    ClearFeature{
        feature: FeatureSelector,
        recipient_index: u16,
    },
    SetFeature{
        feature: FeatureSelector,
        test_mode: u8,
        recipient_index: u16,
    },
    SetAddress{
        device_address: u16,
    },
    GetDescriptor{
        descriptor_type: DescriptorType,
        descriptor_index: u8,
        lang_id: u16,
    },
    SetDescriptor{
        descriptor_type: DescriptorType,
        descriptor_index: u8,
        lang_id: u16,
        descriptor_length: u16,
    },
    GetConfiguration,
    SetConfiguration{
        configuration: u8,
    },
    GetInterface{
        interface: u16,
    },
    SetInterface,
    SynchFrame,
}

#[derive(Debug)]
pub enum DescriptorType {
    Device = 1,
    Configuration,
    String,
    Interface,
    Endpoint,
    DeviceQualifier,
    OtherSpeedConfiguration,
    InterfacePower,
}

fn get_descriptor_type(byte: u8) -> Option<DescriptorType> {
    match byte {
        1 => Some(DescriptorType::Device),
        2 => Some(DescriptorType::Configuration),
        3 => Some(DescriptorType::String),
        4 => Some(DescriptorType::Interface),
        5 => Some(DescriptorType::Endpoint),
        6 => Some(DescriptorType::DeviceQualifier),
        7 => Some(DescriptorType::OtherSpeedConfiguration),
        8 => Some(DescriptorType::InterfacePower),
        _ => None,
    }
}

// Get a descriptor type that is legal in a SetDescriptor request
fn get_set_descriptor_type(byte: u8) -> Option<DescriptorType> {
    match get_descriptor_type(byte) {
        dt @ Some(DescriptorType::Device) => dt,
        dt @ Some(DescriptorType::Configuration) => dt,
        dt @ Some(DescriptorType::String) => dt,
        _ => None
    }
}

#[derive(Copy, Clone)]
pub struct DeviceRequestType(u8);

impl DeviceRequestType {
    pub fn transfer_direction(self) -> TransferDirection {
        match self.0 & (1 << 7) {
            0 => TransferDirection::HostToDevice,
            _ => TransferDirection::DeviceToHost,
        }
    }

    pub fn request_type(self) -> RequestType {
        match (self.0 & (0b11 << 5)) >> 5 {
            0 => RequestType::Standard,
            1 => RequestType::Class,
            2 => RequestType::Vendor,
            _ => RequestType::Reserved,
        }
    }

    pub fn recipient(self) -> Recipient {
        match self.0 & 0b11111 {
            0 => Recipient::Device,
            1 => Recipient::Interface,
            2 => Recipient::Endpoint,
            3 => Recipient::Other,
            _ => Recipient::Reserved,
        }
    }
}

impl fmt::Debug for DeviceRequestType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{{:?}, {:?}, {:?}}}",
               self.transfer_direction(), self.request_type(), self.recipient())
    }
}

#[derive(Debug)]
pub enum TransferDirection {
    DeviceToHost,
    HostToDevice,
}

#[derive(Debug)]
pub enum RequestType {
    Standard,
    Class,
    Vendor,
    Reserved,
}

#[derive(Debug)]
pub enum Recipient {
    Device,
    Interface,
    Endpoint,
    Other,
    Reserved,
}

#[derive(Debug)]
pub enum FeatureSelector {
    DeviceRemoteWakeup,
    EndpointHalt,
    TestMode,
    Unknown,
}

impl FeatureSelector {
    fn get(value: u16) -> Self {
        match value {
            1 => FeatureSelector::DeviceRemoteWakeup,
            0 => FeatureSelector::EndpointHalt,
            2 => FeatureSelector::TestMode,
            _ => FeatureSelector::Unknown,
        }
    }
}

pub trait Descriptor {
    fn size() -> usize;
}

pub struct ConfigurationDescriptor<'a>(&'a [u8]);

impl<'a> Descriptor for ConfigurationDescriptor<'a> {
    fn size() -> usize { 9 }
}

impl<'a> ConfigurationDescriptor<'a> {
    pub fn as_bytes(&self) -> &'a [u8] {
        self.0
    }

    pub fn place(buf: &'a mut [u8],
                 num_interfaces: u8,
                 configuration_value: u8,
                 string_index: u8,
                 attributes: ConfigurationAttributes,
                 max_power: u8,   // in 2mA units
                 related_descriptor_length: usize,
                 ) -> Self {

        if buf.len() < 9 {
            panic!("Not enough room to allocate");
        }

        // Deposit the descriptor at the end of the provided buffer
        let b = &mut buf[buf.len() - 9 ..];

        b[0] = 9; // Size of descriptor
        b[1] = DescriptorType::Configuration as u8;
        put_u16(&mut b[2..4], (9 + related_descriptor_length) as u16);
        b[4] = num_interfaces;
        b[5] = configuration_value;
        b[6] = string_index;
        b[7] = From::from(attributes);
        b[8] = max_power;

        ConfigurationDescriptor(b)
    }
}

fn put_u16<'a>(buf: &'a mut [u8], n: u16) {
    if buf.len() != 2 {
        panic!("Wrong length");
    }
    buf[0] = (n & 0xff) as u8;
    buf[1] = (n >> 8) as u8;
}

pub struct ConfigurationAttributes(u8);

impl ConfigurationAttributes {
    pub fn new(is_self_powered: bool, supports_remote_wakeup: bool) -> Self {
        ConfigurationAttributes(if is_self_powered { 1 << 6 } else { 0 }
                                | if supports_remote_wakeup { 1 << 5 } else { 0 })
    }
}

impl From<ConfigurationAttributes> for u8 {
    fn from(ca: ConfigurationAttributes) -> u8 {
        ca.0
    }
}
