use usbc::common_register::*;

#[derive(Copy, Clone, PartialEq)]
pub enum Mode {
    Host,
    Device(Speed),
}

// value for USBCON.UIMOD
impl ToWord for Mode {
    fn to_word(self) -> u32 {
        match self {
            Mode::Host => 0,
            Mode::Device(_) => 1,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum Speed {
    Full,
    Low,
}

impl ToWord for Speed {
    fn to_word(self) -> u32 {
        match self {
            Speed::Full => 0,
            Speed::Low => 1,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum State {
    Reset,
    Idle(Mode),
    Active(Mode),
}

#[repr(C, packed)]
pub struct Endpoint {
    bank0: Bank,
    bank1: Bank,
}

#[repr(C, packed)]
pub struct Bank {
    addr: Buffer,
    packet_size: PacketSize,
    ctrl_status: ControlStatus,
}

#[repr(C, packed)]
pub struct Buffer(u32);

#[repr(C, packed)]
pub struct PacketSize(u32);

impl PacketSize {
    pub fn new(byte_count: u32, multi_packet_size: u32, auto_zlp: bool) -> PacketSize {
        PacketSize((byte_count & 0x7ffff) |
                   ((multi_packet_size & 0x7ffff) << 16) |
                   ((if auto_zlp { 1 } else { 0 }) << 31))
    }

    pub fn byte_count(&self) -> u32 {
        self.0 & 0x7fff
    }

    pub fn multi_packet_size(&self) -> u32 {
        (self.0 >> 16) & 0x7fff
    }

    pub fn auto_zlp(&self) -> bool {
        self.0 & (1 << 31) != 0
    }
}

#[repr(C, packed)]
pub struct ControlStatus(u32);

impl ControlStatus {
    // Stall request for next transfer
    fn set_stallreq_next() { }

    fn get_status_underflow(&self) -> bool {
        self.0 & (1 << 18) != 0
    }

    fn get_status_overflow(&self) -> bool {
        self.0 & (1 << 17) != 0
    }

    fn get_status_crcerror(&self) -> bool {
        self.0 & (1 << 16) != 0
    }
}
