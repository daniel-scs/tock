// see "41. Cyclic Redundancy Check Calculation Unit (CRCCU)"

// Notes
//
// http://www.at91.com/discussions/viewtopic.php/f,29/t,24859.html
//      Atmel is using the low bit instead of the high bit so reversing
//      the values before calculation did the trick. Here is a calculator
//      that matches (click CCITT and check the 'reverse data bytes' to
//      get the correct value).  http://www.zorc.breitbandkatze.de/crc.html
//
//      The SAM4L calculates 0x1541 for "ABCDEFG".

use core::cell::Cell;
use kernel::hil::crc::{CRC, Client};
use pm::{Clock, HSBClock, PBBClock, enable_clock};

// see "7.1 Product Mapping"
const CRCCU_BASE: u32 = 0x400A4000;

struct Reg(*mut u32);

impl Reg {
    fn read(self) -> u32 {
        unsafe { ::core::ptr::read_volatile(self.0) }
    }

    fn write(self, n: u32) {
        unsafe { ::core::ptr::write_volatile(self.0, n); }
    }
}

// The following macro expands a list of expressions like this:
//
//    { 0x00, "Descriptor Base Register", DSCR, "RW" },
//
// into a series of items like this:
//
//    const DSCR: Reg = Reg((CRCCU_BASE + 0x00) as *mut u32);

macro_rules! registers {
    [ $( { $offset:expr, $description:expr, $name:ident, $access:expr } ),* ] => {
        $( const $name: Reg = Reg((CRCCU_BASE + $offset) as *mut u32); )*
    };
}

// from Table 41.1 in Section 41.6:
registers![
    { 0x00, "Descriptor Base Register", DSCR, "RW" },        // Address of descriptor (512-byte aligned)
    { 0x08, "DMA Enable Register", DMAEN, "W" },             // Write a one to enable DMA channel
    { 0x0C, "DMA Disable Register", DMADIS, "W" },           // Write a one to disable DMA channel
    { 0x10, "DMA Status Register", DMASR, "R" },             // DMA channel enabled?
    { 0x14, "DMA Interrupt Enable Register", DMAIER, "W" },  // Write a one to enable DMA interrupt
    { 0x18, "DMA Interrupt Disable Register", DMAIDR, "W" }, // Write a one to disable DMA interrupt
    { 0x1C, "DMA Interrupt Mask Register", DMAIMR, "R" },    // DMA interrupt enabled?
    { 0x20, "DMA Interrupt Status Register", DMAISR, "R" },  // DMA transfer completed? (cleared when read)
    { 0x34, "Control Register", CR, "W" },                   // Write a one to reset SR
    { 0x38, "Mode Register", MR, "RW" },                     // Bandwidth divider, Polynomial type, Compare?, Enable?
    { 0x3C, "Status Register", SR, "R" },                    // CRC result (unreadable if MR.COMPARE=1)
    { 0x40, "Interrupt Enable Register", IER, "W" },         // Write one to set ERR bit in IMR (zero no effect)
    { 0x44, "Interrupt Disable Register", IDR, "W" },        // Write zero to clear ERR bit in IMR (one no effect)
    { 0x48, "Interrupt Mask Register", IMR, "R" },           // If ERR bit is set, error-interrupt (for compare) is enabled
    { 0x4C, "Interrupt Status Register", ISR, "R" },         // CRC error (for compare)? (cleared when read)
    { 0xFC, "Version Register", VERSION, "R" }               // 12 low-order bits: version of this module.  = 0x00000202
];

// A datatype for forcing alignment
#[repr(simd)]
struct FiveTwelveBytes(
    u64, u64, u64, u64, u64, u64, u64, u64,
    u64, u64, u64, u64, u64, u64, u64, u64,
    u64, u64, u64, u64, u64, u64, u64, u64,
    u64, u64, u64, u64, u64, u64, u64, u64,
    u64, u64, u64, u64, u64, u64, u64, u64,
    u64, u64, u64, u64, u64, u64, u64, u64,
    u64, u64, u64, u64, u64, u64, u64, u64,
    u64, u64, u64, u64, u64, u64, u64, u64,
);

#[repr(C, packed)]
struct Descriptor {
    // Ensure that Descriptor is 512-byte aligned, as required by hardware
    _align: [FiveTwelveBytes; 0],
    addr: u32,       // Transfer Address Register (RW): Address of memory block to compute
    ctrl: TCR,       // Transfer Control Register (RW): IEN, TRWIDTH, BTSIZE
    _res: [u32; 2],
    crc: u32         // Transfer Reference Register (RW): Reference CRC (for compare mode)
}

impl Descriptor {
    const fn new() -> Self {
        Descriptor { addr: 0, ctrl: TCR::default(), crc: 0, _res: [0, 0], _align: [] }
    }
}

#[repr(C, packed)]
struct TCR(u32);  // see "41.6.18 Transfer Control Register"

impl TCR {
    const fn new(ien: bool, trwidth: TrWidth, btsize: u16) -> Self {
        TCR((ien as u32) << 27
            | (trwidth as u32) << 24
            | (btsize as u32))
    }
    const fn default() -> Self {
        Self::new(false, TrWidth::Byte, 0)
    }
    fn get_ien(&self) -> bool {
        (self.0 & (1 << 27)) != 0
    }
}

pub enum TrWidth { Byte, HalfWord, Word }

// see "41.6.10 Mode Register"
struct Mode(u32);

impl Mode {
	fn new(divider: u8, ptype: Polynomial, compare: bool, enable: bool) -> Self {
        Mode(((divider as u32) & 0xf0)
             | (ptype as u32) << 2
             | (compare as u32) << 1
             | (enable as u32))
    }
}

pub enum Polynomial {
	CCIT8023,   // Polynomial 0x04C11DB7
	CASTAGNOLI, // Polynomial 0x1EDC6F41
	CCIT16,		// Polynomial 0x1021
}

pub struct Crccu<'a> {
    descriptor: Descriptor,
    client: Cell<Option<&'a Client>>,
}

impl<'a> Crccu<'a> {
    const fn new() -> Self {
        Crccu { descriptor: Descriptor::new(),
                client: Cell::new(None) }
    }

    pub fn set_client(&self, client: &'a Client) {
        self.client.set(Some(client));
    }

    pub fn handle_interrupt(&mut self) {
        if DMAISR.read() & 1 == 1 {
            // A DMA transfer has completed

            if self.descriptor.ctrl.get_ien() {
                if let Some(client) = self.client.get() {
                    let result = SR.read();
                    client.receive_result(result);
                }

                // Disable the unit
                let enable = false;
                let mode = Mode::new(0, Polynomial::CCIT8023, false, enable);
                MR.write(mode.0);

                // Clear CTRL.IEN (for our own statekeeping)
                self.descriptor.ctrl = TCR::default();
                
                // Disable DMA interrupt and DMA channel
                DMAIDR.write(1);
                DMADIS.write(1);
            }

            /*
            unsafe {
                disable_clock(Clock::PBB(PBBClock::CRCCU));
                disable_clock(Clock::HSB(HSBClock::CRCCU));
            }
            */
        }

        if ISR.read() & 1 == 1 {
            // A CRC error has occurred
        }
    }
}

impl<'a> CRC for Crccu<'a> {
    fn get_version() -> u32 {
        VERSION.read()
    }

    fn compute(&mut self, data: &[u8]) -> bool {
        if self.descriptor.ctrl.get_ien() {
            return false;   // A computation is already in progress
        }

        if data.len() > (2^16 - 1) {
            return false; // Buffer to long (TODO: chain CRCCU computations for large buffers)
        }

        unsafe {
            // see "10.7.4 Clock Mask"
            enable_clock(Clock::HSB(HSBClock::CRCCU));
            enable_clock(Clock::PBB(PBBClock::CRCCU));
        }

        self.descriptor.addr = data.as_ptr() as u32;
        self.descriptor.ctrl = TCR::new(true, TrWidth::Byte, data.len() as u16);
        DSCR.write(&self.descriptor as *const Descriptor as u32);

        CR.write(1);  // Reset intermediate CRC value

        // Enable DMA interrupt and DMA channel
        DMAIER.write(1);
        DMAEN.write(1);

        // Configure the unit to compute a checksum
        let divider = 0;
        let compare = false;
        let enable = true;
        let mode = Mode::new(divider, Polynomial::CCIT8023, compare, enable);
        MR.write(mode.0);

        return true;
    }
}

pub static mut CRCCU: Crccu<'static> = Crccu::new();

// Provide a default interrupt handler
use nvic;
interrupt_handler!(interrupt_handler, CRCCU);
