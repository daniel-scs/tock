// see ???
const CRCCU_BASE: u32 = 0x400A4000;

    // XXX: see "10.7.4 Clock Mask" about enabling the CRCCU clock
    // XXX: see "15.6 Module Configuration"
    //      see "41. Cyclic Redundancy Check Calculation Unit (CRCCU)"

    // "The CRCCU interrupt request line is connected to the NVIC. Using the CRCCU interrupt
    // requires the NVIC to be configured first."

    // Write CR.RESET=1 to reset intermediate CRC value
    // Write MR.ENABLE=1 to perform checksum
    // Configure MR.PTYPE to choose algorithm

struct Reg(*mut u32);
unsafe impl Sync for Reg { }

// The following macro expands a list of expressions like this:
//
//    { 0x00, "Descriptor Base Register", DSCR, "RW" }
// 
// into a series of items like this:
//
//    static DSCR: Reg = Reg((CRCCU_BASE + 0x00) as *mut u32);

macro_rules! registers {
    [ $( { $offset:expr, $description:expr, $name:ident, $access:expr } ),* ] => {
        $( static $name: Reg = Reg((CRCCU_BASE + $offset) as *mut u32); )*
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

// must be 512-byte aligned
#[repr(C, packed)]
struct Descriptor {
    addr: u32,       // Transfer Address Register (RW): Address of memory block to compute
    ctrl: u32,       // Transfer Control Register (RW): IEN, TRWIDTH, BTSIZE
    _res: [u32; 2],
    crc: u32         // Transfer Reference Register (RW): Reference CRC (for compare mode)
}
