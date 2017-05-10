use core::fmt;

pub trait Client {
    fn bus_reset(&self);

    fn received_setup(&self /* , descriptor/bank */);

    fn received_out(&self /* , descriptor/bank */);
}

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
    pub fn standard_request_type(&self) -> Option<StandardDeviceRequest> {
        match self.request_type.request_type() {
            RequestType::Standard =>
                match self.request_code {
                    5 => Some(StandardDeviceRequest::SetAddress{
                            device_address: self.value
                         }),
                    6 => {
                        if let Some(dt) = get_descriptor_type((self.value >> 8) as u8) {
                            Some(StandardDeviceRequest::GetDescriptor{
                                    descriptor_type: dt,
                                    descriptor_index: (self.value & 0xff) as u8,
                                    lang_id: self.index,
                                 })
                        }
                        else {
                            None
                        }
                    }
                    _ => None,
                },
            _ => None,
        }
    }
}

pub fn get_u16(buf: &[u8]) -> Option<u16> {
    if buf.len() != 2 {
        return None;
    }
    Some ((buf[0] as u16) | ((buf[1] as u16) << 8))
}

#[derive(Debug)]
pub enum StandardDeviceRequest {
    SetAddress{
        device_address: u16,
    },
    GetDescriptor{
        descriptor_type: DescriptorType,
        descriptor_index: u8,
        lang_id: u16,
    }
}

#[derive(Debug)]
pub enum DescriptorType {
    Device,
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
