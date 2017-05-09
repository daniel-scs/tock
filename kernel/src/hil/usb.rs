pub trait Client {
    fn bus_reset(&self);

    fn received_setup(&self /* , descriptor/bank */);

    fn received_out(&self /* , descriptor/bank */);
}

#[repr(C, packed)]
pub struct SetupData {
    pub request_type: DeviceRequestType,
    pub request_code: u8,
    pub value: u16,
    pub index: u16,
    pub length: u16,
}

impl SetupData {
    pub fn standard_request_type(self) -> Option<StandardDeviceRequest> {
        if self.request_type.request_type() != RequestType::Standard {
            return None;
        }
        match self.request_code {
            0b10000000 =>
                Some(StandardDeviceRequest::GetDescriptor{
                        descriptor_type: From::from(self.value >> 8 as u8),
                        descriptor_index: self.value & 0xff as u8,
                        lang_id: self.index,
                     }),
            _ => None,
        }
    }
}

pub enum StandardDeviceRequest {
    GetDescriptor{
        descriptor_type: DescriptorType,
        descriptor_index: u8,
        lang_id: u16
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

impl From<u8> for DescriptorType {
    fn from(byte: u8) -> Self {
        match byte {
            0 => DescriptorType::Device,
            1 => DescriptorType::Configuration,
            2 => DescriptorType::String,
            3 => DescriptorType::Interface,
            4 => DescriptorType::Endpoint,
            5 => DescriptorType::DeviceQualifier,
            6 => DescriptorType::OtherSpeedConfiguration,
            7 => DescriptorType::InterfacePower,
        }
    }
}

pub struct DeviceRequestType(u8);

impl DeviceRequestType {
    pub fn transfer_direction(self) -> TransferDirection {
        match self & (1 << 7) {
            0 => TransferDirection:HostToDevice,
            _ => TransferDirection:DeviceToHost,
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
