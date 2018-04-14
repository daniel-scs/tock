//! Datastructures for manipulating the SAM4L USB Controller

use core::fmt;
use core::ptr;
use kernel::common::VolatileCell;

pub const N_ENDPOINTS: usize = 8;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum State {
    Reset,
    Idle(Mode),
    Active(Mode),
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Mode {
    Host,
    Device {
        speed: Speed,
        config: DeviceConfig,
        state: DeviceState,
    },
}

impl Mode {
    pub fn device_at_speed(speed: Speed) -> Mode {
        Mode::Device {
            speed: speed,
            config: Default::default(),
            state: Default::default(),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct DeviceConfig {
    pub endpoint_configs: [Option<EndpointConfig>; N_ENDPOINTS],
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct DeviceState {
    pub endpoint_states: [EndpointState; N_ENDPOINTS]
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum EndpointState {
    Disabled,
    Init,
    CtrlReadIn,
    CtrlReadStatus,
    CtrlWriteOut,
    CtrlWriteStatus,
    CtrlWriteStatusWait,
    CtrlInDelay,
}

impl Default for EndpointState {
    fn default() -> Self {
        EndpointState::Disabled
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Speed {
    Full,
    Low,
}

pub type Endpoint = [Bank; 2];

pub const fn new_endpoint() -> Endpoint {
    [Bank::new(), Bank::new()]
}

#[repr(C)]
pub struct Bank {
    pub addr: VolatileCell<*mut u8>,
    pub packet_size: VolatileCell<PacketSize>,
    pub ctrl_status: VolatileCell<ControlStatus>,
    _pad: u32,
}

impl Bank {
    pub const fn new() -> Bank {
        Bank {
            addr: VolatileCell::new(ptr::null_mut()),
            packet_size: VolatileCell::new(PacketSize(0)),
            ctrl_status: VolatileCell::new(ControlStatus(0)),
            _pad: 0,
        }
    }

    pub fn set_addr(&self, addr: *mut u8) {
        self.addr.set(addr);
    }

    pub fn set_packet_size(&self, size: PacketSize) {
        self.packet_size.set(size);
    }
}

pub enum BankIndex {
    Bank0,
    Bank1,
}

impl From<BankIndex> for usize {
    fn from(bi: BankIndex) -> usize {
        match bi {
            BankIndex::Bank0 => 0,
            BankIndex::Bank1 => 1,
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PacketSize(u32);

impl PacketSize {
    pub fn new(byte_count: u32, multi_packet_size: u32, auto_zlp: bool) -> PacketSize {
        PacketSize(
            (byte_count & 0x7fff) | ((multi_packet_size & 0x7fff) << 16) | ((if auto_zlp {
                1 << 31
            } else {
                0
            })),
        )
    }

    pub fn default() -> PacketSize {
        PacketSize::new(0, 0, false)
    }

    pub fn single(byte_count: u32) -> PacketSize {
        PacketSize::new(byte_count, 0, false)
    }

    pub fn single_with_zlp(byte_count: u32) -> PacketSize {
        PacketSize::new(byte_count, 0, true)
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
        write!(
            f,
            "PacketSize {:x} {{ byte_count: {}, multi_packet_size: {}, {}auto_zlp }}",
            self.0,
            self.byte_count(),
            self.multi_packet_size(),
            bang(self.auto_zlp())
        )
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ControlStatus(u32);

impl ControlStatus {
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
        write!(
            f,
            "ControlStatus {:x} {{ {}underflow {}overflow {}crcerror }}",
            self.0,
            bang(self.get_status_underflow()),
            bang(self.get_status_overflow()),
            bang(self.get_status_crcerror())
        )
    }
}

fn bang(b: bool) -> &'static str {
    if b {
        ""
    } else {
        "!"
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C)]
pub struct EndpointConfig(u32);

impl EndpointConfig {
    /// Create an endpoint configuration
    pub fn new(
        banks: BankCount,
        size: EndpointSize,
        dir: EndpointDirection,
        typ: EndpointType,
        redir: EndpointIndex,
    ) -> EndpointConfig {
        EndpointConfig(
            ((banks as u32) << 2) | ((size as u32) << 4) | ((dir as u32) << 8)
                | ((typ as u32) << 11) | (redir.to_u32() << 16),
        )
    }
}

impl From<EndpointConfig> for u32 {
    fn from(epc: EndpointConfig) -> u32 {
        epc.0
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

pub struct EndpointIndex(u8);

impl EndpointIndex {
    pub fn new(index: u32) -> EndpointIndex {
        EndpointIndex(index as u8 & 0xf)
    }

    pub fn to_u32(self) -> u32 {
        self.0 as u32
    }
}

impl From<EndpointIndex> for usize {
    fn from(ei: EndpointIndex) -> usize {
        ei.0 as usize
    }
}

pub struct HexBuf<'a>(pub &'a [u8]);

impl<'a> fmt::Debug for HexBuf<'a> {
    #[allow(unused_must_use)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[");
        let mut i: usize = 0;
        for b in self.0 {
            write!(f, "{}{:.02x}", if i > 0 { " " } else { "" }, b);
            i += 1;
        }
        write!(f, "]")
    }
}
