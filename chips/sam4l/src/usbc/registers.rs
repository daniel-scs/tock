//! Registers of the SAM4L's USB controller

// Bitfields for UDINT, UDINTCLR, UDINTESET
pub const UDINT_SUSP: u32 = 1 << 0;
pub const UDINT_SOF: u32 = 1 << 2;
pub const UDINT_EORST: u32 = 1 << 3;
pub const UDINT_WAKEUP: u32 = 1 << 4;
pub const UDINT_EORSM: u32 = 1 << 5;
pub const UDINT_UPRSM: u32 = 1 << 6;

// Bitfields for UECONnSET, UESTAn, UESTAnCLR
pub const TXIN: u32 = 1 << 0;
pub const RXOUT: u32 = 1 << 1;
pub const RXSTP: u32 = 1 << 2;
pub const ERRORF: u32 = 1 << 2;
pub const NAKOUT: u32 = 1 << 3;
pub const NAKIN: u32 = 1 << 4;
pub const STALLED: u32 = 1 << 6;
pub const CRCERR: u32 = 1 << 6;
pub const RAMACERR: u32 = 1 << 11;
pub const STALLRQ: u32 = 1 << 19;

// Bitfields for UESTAn
pub const CTRLDIR: u32 = 1 << 17;
