// see "41. Cyclic Redundancy Check Calculation Unit (CRCCU)"

    // TODO:
    // see "10.7.4 Clock Mask": enable the CRCCU clock by setting HSBMASK.4, PBBMASK.4
    // see "15.6 Module Configuration"

    // Write CR.RESET=1 to reset intermediate CRC value
    // Configure MR.PTYPE to choose algorithm
    // Write MR.ENABLE=1 to perform checksum

    // crc calculator: http://www.zorc.breitbandkatze.de/crc.html
    //
    // "Atmel is using the low bit instead of the high bit so reversing the values before
    // calculation did the trick. Here is a calculator that matches (click CCITT and check the
    // 'reverse data bytes' to get the correct value)."
    // "The SAM4L calculates 0x1541 for ABCDEFG"
    //      http://www.at91.com/discussions/viewtopic.php/f,29/t,24859.html

use core::cell::Cell;
use kernel::hil::crc;
use pm::{Clock, HSBClock, PBBClock, enable_clock, disable_clock};

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
    { 0x40, "Interrupt Enable Register", IER, "W" },         // Write ones to set bits in IMR (zeros no effect)
    { 0x44, "Interrupt Disable Register", IDR, "W" },        // Write zeros to clear bits in IMR (ones no effect)
    { 0x48, "Interrupt Mask Register", IMR, "R" },           // Bit set means interrupt enabled
    { 0x4C, "Interrupt Status Register", ISR, "R" },         // CRC error? (cleared when read)
    { 0xFC, "Version Register", VERSION, "R" }               // 12 low-order bits: version of this module
];

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

// must be 512-byte aligned
#[repr(C, packed)]
struct Descriptor {
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
struct TCR(u32);

impl TCR {
    const fn new(ien: bool, trwidth: TrWidth, btsize: u16) -> Self {
        TCR((ien as u32) << 27
            | (trwidth as u32) << 24
            | btsize as u32)
    }
    const fn default() -> Self {
        Self::new(false, TrWidth::Byte, 0)
    }
}

enum TrWidth { Byte, HalfWord, Word }

pub struct Crccu<'a> {
    descriptor: Descriptor,
    client: Cell<Option<&'a crc::Client>>,
}

impl<'a> Crccu<'a> {
    const fn new() -> Self {
        Crccu { descriptor: Descriptor::new(),
                client: Cell::new(None) }
    }

    pub fn set_client(&self, client: &'a crc::Client) {
        self.client.set(Some(client));
    }

    pub fn handle_interrupt(&self) {
    }

    /*
    enable_clock(Clock::HSB(HSBClock::CRCCU));
    enable_clock(Clock::PBB(PBBClock::CRCCU));
    */
}

impl<'a> crc::CRC for Crccu<'a> {
    fn get_version() -> u32 {
        VERSION.read()
    }
}

pub static mut CRCCU: Crccu<'static> = Crccu::new();

// make a default interrupt handler
use nvic;
interrupt_handler!(interrupt_handler, CRCCU);
