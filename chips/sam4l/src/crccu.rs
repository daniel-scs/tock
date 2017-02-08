pub static CRCCU_BASE: u32 = 0x400A4000;

// XXX: see "10.7.4 Clock Mask" about enabling the CRCCU clock
// XXX: see "15.6 Module Configuration"
// see "41. Cyclic Redundancy Check Calculation Unit (CRCCU)"

// "The CRCCU interrupt request line is connected to the NVIC. Using the CRCCU interrupt
// requires the NVIC to be configured first."

// Write CR.RESET=1 to reset intermediate CRC value
// Write MR.ENABLE=1 to perform checksum
// Configure MR.PTYPE to choose algorithm

pub static CRCCU_DSCR: u32 = CRCCU_BASE + 0;

#[repr(C)]
pub struct crccu_reg {

}
