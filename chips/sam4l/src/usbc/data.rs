use core::cell::Cell;
use core::fmt;
use core::ptr;
use usbc::common_register::*;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Mode {
    Host,
    Device { speed: Speed,
             config: Option<EndpointConfig>,
             state: DeviceState,
           },
}

impl Mode {
    pub fn device_at_speed(speed: Speed) -> Mode {
        Mode::Device{ speed: speed, config: None, state: DeviceState::Init }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum DeviceState {
    Init,
    SetupIn,
    SetupOut,
}

// value for USBCON.UIMOD
impl ToWord for Mode {
    fn to_word(self) -> u32 {
        match self {
            Mode::Host => 0,
            Mode::Device{ .. } => 1,
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

pub type Endpoint = [Bank; 2];

pub const fn new_endpoint() -> Endpoint {
    [Bank::new(), Bank::new()]
}

#[repr(C, packed)]
pub struct Bank {
    pub addr: Cell<Buffer>,
    pub packet_size: Cell<PacketSize>,
    pub ctrl_status: Cell<ControlStatus>,
}

impl Bank {
    pub const fn new() -> Bank {
        Bank { addr: Cell::new(Buffer(ptr::null_mut())),
               packet_size: Cell::new(PacketSize(0)),
               ctrl_status: Cell::new(ControlStatus(0)) }
    }

    pub fn set_addr(&self, addr: Buffer) {
        self.addr.set(addr);
    }

    pub fn set_packet_size(&self, size: PacketSize) {
        self.packet_size.set(size);
    }
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Buffer(pub *mut u8);

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct PacketSize(u32);

impl PacketSize {
    pub fn new(byte_count: u32, multi_packet_size: u32, auto_zlp: bool) -> PacketSize {
        PacketSize((byte_count & 0x7ffff) |
                   ((multi_packet_size & 0x7ffff) << 16) |
                   ((if auto_zlp { 1 } else { 0 }) << 31))
    }

    pub fn single(byte_count: u32) -> PacketSize {
        PacketSize::new(byte_count, 0, false)
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

impl fmt::Debug for PacketSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PacketSize {:08x} {{ byte_count: {}, multi_packet_size: {}, auto_zlp: {} }}",
               self.0, self.byte_count(), self.multi_packet_size(), self.auto_zlp())
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

impl fmt::Debug for ControlStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ControlStatus {:08x} {{ underflow: {}, overflow: {}, crcerror: {} }}",
               self.0, self.get_status_underflow(), self.get_status_overflow(), self.get_status_crcerror())
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
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
