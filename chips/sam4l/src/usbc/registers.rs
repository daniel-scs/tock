#![allow(non_upper_case_globals)]

use core::ops::{BitOr, Not};
use usbc::common_register::*;

// Base address of USBC registers.  See "7.1 Product Mapping"
const USBC_BASE: u32 = 0x400A5000;

// USBCON

#[derive(Copy, Clone)]
pub struct UsbCon(pub u32);

impl BitOr for UsbCon {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        UsbCon(self.0 | rhs.0)
    }
}

impl Not for UsbCon {
    type Output = Self;
    fn not(self) -> Self {
        UsbCon(!self.0)
    }
}

impl FromWord for UsbCon {
    fn from_word(n: u32) -> Self {
        UsbCon(n)
    }
}

impl ToWord for UsbCon {
    fn to_word(self) -> u32 {
        self.0
    }
}

pub const UIMOD: UsbCon = UsbCon(1 << 25);
pub const USBE: UsbCon = UsbCon(1 << 15);
pub const FRZCLK: UsbCon = UsbCon(1 << 14);

reg![0x0000, "Device General Control Register", UDCON, "RW"];
reg![0x0004, "Device Global Interrupt Register", UDINT, "R"];
reg![0x0008, "Device Global Interrupt Clear Register", UDINTCLR, "W"];
reg![0x000C, "Device Global Interrupt Set Register", UDINTSET, "W"];
reg![0x0010, "Device Global Interrupt Enable Register", UDINTE, "R"];
reg![0x0014, "Device Global Interrupt Enable Clear Register", UDINTECLR, "W"];
reg![0x0018, "Device Global Interrupt Enable Set Register", UDINTESET, "W"];
reg![0x001C, "Endpoint Enable/Reset Register", UERST, "RW"];
reg![0x0020, "Device Frame Number Register", UDFNUM, "R"];

regs![0x0100, "Endpoint n Configuration Register", UECFGn, "RW", 8];
regs![0x0130, "Endpoint n Status Register", UESTAn, "R", 8];
regs![0x0160, "Endpoint n Status Clear Register", UESTAnCLR, "W", 8];
regs![0x0190, "Endpoint n Status Set Register", UESTAnSET, "W", 8];
regs![0x01C0, "Endpoint n Control Register", UECONn, "R", 8];
regs![0x01F0, "Endpoint n Control Set Register", UECONnSET, "W", 8];
regs![0x0220, "Endpoint n Control Clear Register", UECONnCLR, "W", 8];
 
reg![0x0400, "Host General Control Register", UHCON, "RW"];
reg![0x0404, "Host Global Interrupt Register", UHINT, "R"];
reg![0x0408, "Host Global Interrupt Clear Register", UHINTCLR, "W"];
reg![0x040C, "Host Global Interrupt Set Register", UHINTSET, "W"];
reg![0x0410, "Host Global Interrupt Enable Register", UHINTE, "R"];
reg![0x0414, "Host Global Interrupt Enable Clear Register", UHINTECLR, "W"];
reg![0x0418, "Host Global Interrupt Enable Set Register", UHINTESET, "W"];
reg![0x041C, "Pipe Enable/Reset Register", UPRST, "RW"];
reg![0x0420, "Host Frame Number Register", UHFNUM, "RW"];
reg![0x0424, "Host Start Of Frame Control Register", UHSOFC, "RW"];

regs![0x0500, "Pipe n Configuration Register", UPCFGn, "RW", 8];
regs![0x0530, "Pipe n Status Register", UPSTAn, "R", 8];
regs![0x0560, "Pipe n Status Clear Register", UPSTAnCLR, "W", 8];
regs![0x0590, "Pipe n Status Set Register", UPSTAnSET, "W", 8];
regs![0x05C0, "Pipe n Control Register", UPCONn, "R", 8];
regs![0x05F0, "Pipe n Control Set Register", UPCONnSET, "W", 8];
regs![0x0620, "Pipe n Control Clear Register", UPCONnCLR, "W", 8];
regs![0x0650, "Pipe n IN Request Register", UPINRQn, "RW", 8];

reg![0x0800, "General Control Register", USBCON, "RW", UsbCon ];
reg![0x0804, "General Status Register", USBSTA, "R"];
reg![0x0808, "General Status Clear Register", USBSTACLR, "W"];
reg![0x080C, "General Status Set Register", USBSTASET, "W"];
reg![0x0818, "IP Version Register", UVERS, "R"];
reg![0x081C, "IP Features Register", UFEATURES, "R"];
reg![0x0820, "IP PB Address Size Register", UADDRSIZE, "R"];
reg![0x0824, "IP Name Register 1", UNAME1, "R"];
reg![0x0828, "IP Name Register 2", UNAME2, "R"];
reg![0x082C, "USB Finite State Machine Status Register", USBFSM, "R"];
reg![0x0830, "USB Descriptor address", UDESC, "RW"];

// bitfield![UDCON_DETACH, UDCON, bool, 1, 8]
pub const UDCON_DETACH: BitField<bool> = BitField::new(UDCON, 1, 8);
