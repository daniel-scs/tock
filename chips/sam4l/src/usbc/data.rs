use usbc::common_register::*;

pub type Address = u32; // XXX

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
pub struct EndpointDescriptor {
    addr: u32,
    packet_size: PacketSize,
    ctrl_status: ControlStatus,
}

pub struct ControlStatus(u32);

pub struct PacketSize(u32);

impl ControlStatus {
    // Stall request for next transfer
    fn set_stallreq_next() { }

    fn get_status_underflow(&self) -> bool {
        self.0 & (1 << 18) == 1
    }

    fn get_status_overflow(&self) -> bool {
        self.0 & (1 << 17) == 1
    }

    fn get_status_crcerror(&self) -> bool {
        self.0 & (1 << 16) == 1
    }
}

enum EndpointStatus {
    Underflow,
    Overflow,
    CRCError,
}
