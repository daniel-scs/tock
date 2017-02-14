// see ???
const CRCCU_BASE: u32 = 0x400A4000;

    // XXX: see "10.7.4 Clock Mask" about enabling the CRCCU clock
    // XXX: see "15.6 Module Configuration"
    // see "41. Cyclic Redundancy Check Calculation Unit (CRCCU)"

    // "The CRCCU interrupt request line is connected to the NVIC. Using the CRCCU interrupt
    // requires the NVIC to be configured first."

    // Write CR.RESET=1 to reset intermediate CRC value
    // Write MR.ENABLE=1 to perform checksum
    // Configure MR.PTYPE to choose algorithm

static CRCCU_DSCR: u32 = CRCCU_BASE + 0;

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
    { 0x00, "Descriptor Base Register", DSCR, "RW" },
    { 0x08, "DMA Enable Register", DMAEN, "W" },
    { 0x0C, "DMA Disable Register", DMADIS, "W" },
    { 0x10, "DMA Status Register", DMASR, "R" },
    { 0x14, "DMA Interrupt Enable Register", DMAIER, "W" },
    { 0x18, "DMA Interrupt Disable Register", DMAIDR, "W" },
    { 0x1C, "DMA Interrupt Mask Register", DMAIMR, "R" },
    { 0x20, "DMA Interrupt Status Register", DMAISR, "R" },
    { 0x34, "Control Register", CR, "W" },
    { 0x38, "Mode Register", MR, "RW" },
    { 0x3C, "Status Register", SR, "R" },
    { 0x40, "Interrupt Enable Register", IER, "W" },
    { 0x44, "Interrupt Disable Register", IDR, "W" },
    { 0x48, "Interrupt Mask Register", IMR, "R" },
    { 0x4C, "Interrupt Status Register", ISR, "R" },
    { 0xFC, "Version Register", VERSION, "R" }
];

/*
0x00 Descriptor Base Register DSCR Read-Write
0x08 DMA Enable Register DMAEN Write-only -
0x0C DMA Disable Register DMADIS Write-only -
0x10 DMA Status Register DMASR Read-only 0x00000000
0x14 DMA Interrupt Enable Register DMAIER Write-only -
0x18 DMA Interrupt Disable Register DMAIDR Write-only -
0x1C DMA Interrupt Mask Register DMAIMR Read-only 0x00000000
0x20 DMA Interrupt Status Register DMAISR Read-only 0x00000000
0x34 Control Register CR Write-only -
0x38 Mode Register MR Read/Write 0x00000000
0x3C Status Register SR Read-only 0xFFFFFFFF
0x40 Interrupt Enable Register IER Write-only -
0x44 Interrupt Disable Register IDR Write-only -
0x48 Interrupt Mask Register IMR Read-only 0x00000000
0x4C Interrupt Status Register ISR Read-only 0x00000000
0xFC Version Register VERSION Read-only -(1)
*/
