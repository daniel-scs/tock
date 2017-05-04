
#[repr(C, packed)]
pub struct SetupData {
    pub request_type: DeviceRequestType,
    pub request_code: u8,
    pub value: u16,
    pub index: u16,
    pub length: u16,
}

impl SetupData {
    pub fn standard_request_type(self) -> Option<StandardRequestType> {
        if self.request_type.request_type() != RequestType::Standard {
            return None;
        }
        match self.request_code {
            0b10000000 => {

                Some(StandardDeviceRequest::GetDescriptor),
            }
            _ => None
        }
    }
}

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

impl From<u8> for DescriptorType

pub struct DeviceRequestType(u8);

impl DeviceRequestType {
    pub fn transfer_direction(self) -> TransferDirection {
        match self & (1 << 7) {
            0 => TransferDirection:HostToDevice
            _ => TransferDirection:DeviceToHost
        }
    }

    pub fn request_type(self) -> RequestType {
        match (self & (0b11 << 5)) >> 5 {
            0 => RequestType::Standard,
            1 => RequestType::Class,
            2 => RequestType::Vendor,
            _ => RequestType::Reserved,
        }
    }

    pub fn recipient(self) -> Recipient {
        match self & 0b11111 {
            0 => Recipient::Device,
            1 => Recipient::Interface,
            2 => Recipient::Endpoint,
            3 => Recipient::Other,
            _ => Recipient::Reserved,
        }
    }
}

pub enum TransferDirection {
    DeviceToHost,
    HostToDevice,
}

pub enum RequestType {
    Standard,
    Class,
    Vendor,
    Reserved,
}

pub trait Client {
    fn bus_reset(&self);

    fn received_setup(&self /* , descriptor/bank */);

    fn received_out(&self /* , descriptor/bank */);
}
