//! Registers of the SAM4L's USB controller

#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

use usbc::common_register::*;
use usbc::data::*;
use kernel::common::static_ref::*;
use kernel::common::volatile_cell::*;

// Base address of USBC registers.  See "7.1 Product Mapping"
const USBC_BASE: u32 = 0x400A5000;

reg![0x0000, "Device General Control Register", UDCON, UDCON_T, "RW"];
reg![0x0004, "Device Global Interrupt Register", UDINT, UDINT_T, "R"];
reg![0x0008, "Device Global Interrupt Clear Register", UDINTCLR, UDINTCLR_T, "W"];
reg![0x000C, "Device Global Interrupt Set Register", UDINTSET, UDINTSET_T, "W"];
reg![0x0010, "Device Global Interrupt Enable Register", UDINTE, UDINTE_T, "R"];
reg![0x0014, "Device Global Interrupt Enable Clear Register", UDINTECLR, UDINTECLR_T, "W"];
reg![0x0018, "Device Global Interrupt Enable Set Register", UDINTESET, UDINTESET_T, "W"];
reg![0x001C, "Endpoint Enable/Reset Register", UERST, UERST_T, "RW"];
reg![0x0020, "Device Frame Number Register", UDFNUM, UDFNUM_T, "R"];

reg![0x0100, "DEBUG UECFG0", UECFG0, UECFG0_T, "RW"];
reg![0x01C0, "DEBUG UECON0", UECON0, UDCON0_T, "R"];
reg![0x01F0, "DEBUG UECON0SET", UECON0SET, UECON0SET_T, "W"];

regs![0x0100, "Endpoint n Configuration Register", UECFGn, UECFGn_T, "RW", 8];
regs![0x0130, "Endpoint n Status Register", UESTAn, UESTAn_T, "R", 8];
regs![0x0160, "Endpoint n Status Clear Register", UESTAnCLR, UESTAnCLR_T, "W", 8];
regs![0x0190, "Endpoint n Status Set Register", UESTAnSET, UESTAnSET_T, "W", 8];
regs![0x01C0, "Endpoint n Control Register", UECONn, UECONn_T, "R", 8];
regs![0x01F0, "Endpoint n Control Set Register", UECONnSET, UECONnSET_T, "W", 8];
regs![0x0220, "Endpoint n Control Clear Register", UECONnCLR, UECONnCLR_T, "W", 8];

reg![0x0400, "Host General Control Register", UHCON, UHCON_T, "RW"];
reg![0x0404, "Host Global Interrupt Register", UHINT, UHINT_T, "R"];
reg![0x0408, "Host Global Interrupt Clear Register", UHINTCLR, UHINTCLR_T, "W"];
reg![0x040C, "Host Global Interrupt Set Register", UHINTSET, UHINTSET_T, "W"];
reg![0x0410, "Host Global Interrupt Enable Register", UHINTE, UHINTE_T, "R"];
reg![0x0414, "Host Global Interrupt Enable Clear Register", UHINTECLR, UHINTECLR_T, "W"];
reg![0x0418, "Host Global Interrupt Enable Set Register", UHINTESET, UHINTESET_T, "W"];
reg![0x041C, "Pipe Enable/Reset Register", UPRST, UPRST_T, "RW"];
reg![0x0420, "Host Frame Number Register", UHFNUM, UHFNUM_T, "RW"];
reg![0x0424, "Host Start Of Frame Control Register", UHSOFC, UHSOFC_T, "RW"];

regs![0x0500, "Pipe n Configuration Register", UPCFGn, UPCFGn_T, "RW", 8];
regs![0x0530, "Pipe n Status Register", UPSTAn, UPSTAn_T, "R", 8];
regs![0x0560, "Pipe n Status Clear Register", UPSTAnCLR, UPSTAnCLR_T, "W", 8];
regs![0x0590, "Pipe n Status Set Register", UPSTAnSET, UPSTAnSET_T, "W", 8];
regs![0x05C0, "Pipe n Control Register", UPCONn, UPCONn_T, "R", 8];
regs![0x05F0, "Pipe n Control Set Register", UPCONnSET, UPCONnSET_T, "W", 8];
regs![0x0620, "Pipe n Control Clear Register", UPCONnCLR, UPCONnCLR_T, "W", 8];
regs![0x0650, "Pipe n IN Request Register", UPINRQn, UPINRQn_T, "RW", 8];

reg![0x0800, "General Control Register", USBCON, USBCON_T, "RW"];
reg![0x0804, "General Status Register", USBSTA, USBSTA_T, "R"];
reg![0x0808, "General Status Clear Register", USBSTACLR, USBSTACLR_T, "W"];
reg![0x080C, "General Status Set Register", USBSTASET, USBSTASET_T, "W"];
reg![0x0818, "IP Version Register", UVERS, UVERS_T, "R"];
reg![0x081C, "IP Features Register", UFEATURES, UFEATURES_T, "R"];
reg![0x0820, "IP PB Address Size Register", UADDRSIZE, UADDRSIZE_T, "R"];
reg![0x0824, "IP Name Register 1", UNAME1, UNAME1_T, "R"];
reg![0x0828, "IP Name Register 2", UNAME2, UNAME2_T, "R"];
reg![0x082C, "USB Finite State Machine Status Register", USBFSM, USBFSM_T, "R"];
reg![0x0830, "USB Descriptor address", UDESC, UDESC_T, "RW"];

pub struct USBCON_UIMOD_T(StaticRef<USBCON_T>);

impl USBCON_UIMOD_T {
    const fn new(r: StaticRef<USBCON_T>) -> Self {
        USBCON_UIMOD_T(r)
    }

    pub fn write(self, val: Mode) {
        let w = self.0.read();
        let src_mask = 1;
        let shift = 25;
        let dst_mask = src_mask << shift;
        let val_bits = (val.to_word() & src_mask) << shift;
        self.0.write((w & !dst_mask) | val_bits);
    }
}

pub const USBCON_UIMOD: USBCON_UIMOD_T = USBCON_UIMOD_T::new(USBCON);

/*
impl USBCON_T {
    fn UIMOD_write(&self, val: Mode) {
        let w = UIMOD.read();
        let src_mask = 1;
        let shift = 25;
        let dst_mask = src_mask << shift;
        let val_bits = (val.to_word() & src_mask) << shift;
        self.reg.write((w & !dst_mask) | val_bits);
    }
}
*/

/*
bitfield![USBCON, USBCON_UIMOD, "RW", Mode, 25, 1]; // sheet says bit 25, but maybe it's 24?
bitfield![USBCON, USBCON_USBE, "RW", bool, 15, 1];
bitfield![USBCON, USBCON_FRZCLK, "RW", bool, 14, 1];

bitfield![UDCON, UDCON_DETACH, "RW", bool, 8, 1];
bitfield![UDCON, UDCON_LS, "RW", Speed, 12, 1];
bitfield![UDCON, UDCON_UADD, "RW", u8, 0, 0b1111111];
bitfield![UDCON, UDCON_ADDEN, "RW", bool, 7, 1];

bitfield![USBSTA, USBSTA_CLKUSABLE, "R", bool, 14, 1];
*/

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
