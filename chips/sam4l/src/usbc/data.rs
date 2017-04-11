use core::cell::Cell;
use usbc::common_register::*;

#[derive(Copy, Clone, PartialEq, Debug)]
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

#[derive(Copy, Clone, PartialEq, Debug)]
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

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum State {
    Reset,
    Idle(Mode),
    Active(Mode),
}

#[repr(C, packed)]
pub struct Endpoint {
    banks: [Bank; 2]
}

impl Endpoint {
    pub const fn new() -> Endpoint {
        Endpoint { banks: [Bank::new(), Bank::new()] }
    }
}

#[repr(C, packed)]
pub struct Bank {
    addr: Cell<Buffer>,
    packet_size: Cell<PacketSize>,
    ctrl_status: Cell<ControlStatus>,
}

impl Bank {
    pub const fn new() -> Bank {
        Bank { addr: Cell::new(Buffer(0)),
               packet_size: Cell::new(PacketSize(0)),
               ctrl_status: Cell::new(ControlStatus(0)) }
    }
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Buffer(u32);

#[derive(Copy, Clone)]
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

#[derive(Copy, Clone)]
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

#[repr(C, packed)]
pub struct EndpointConfig(u32);

impl EndpointConfig {
    /// Create an endpoint configuration
    pub fn new(banks: BankCount,
               size: EndpointSize,
               dir: EndpointDirection,
               typ: EndpointType,
               redir: EndpointIndex) -> EndpointConfig {
        EndpointConfig(((banks as u32) << 2) |
                       ((size as u32) << 4) |
                       ((dir as u32) << 8) |
                       ((typ as u32) << 11) |
                       (redir.to_word() << 16))
    }
}

impl ToWord for EndpointConfig {
    fn to_word(self) -> u32 { self.0 }
}

impl FromWord for EndpointConfig {
    fn from_word(_w: u32) -> Self {
        panic!("Unimplemented");
    }
}

pub enum BankCount {
    Single,
    Double,
}

pub enum EndpointSize {
    Bytes8,
    Bytes16,
    Bytes32,
    Bytes64,
    Bytes128,
    Bytes256,
    Bytes512,
    Bytes1024,
}

pub enum EndpointDirection {
    Out,
    In,
}

pub enum EndpointType {
    Control,
    Isochronous,
    Bulk,
    Interrupt,
}

pub struct EndpointIndex(u32);

impl EndpointIndex {
    pub fn new(index: u32) -> EndpointIndex {
        EndpointIndex(index & 0xf)
    }
}

impl ToWord for EndpointIndex {
    fn to_word(self) -> u32 { self.0 }
}
