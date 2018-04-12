use core::fmt;

// Bitfields for UDINT, UDINTCLR, UDINTESET
pub const UDINT_SUSP: u32 = 1 << 0;
pub const UDINT_SOF: u32 = 1 << 2;
pub const UDINT_EORST: u32 = 1 << 3;
pub const UDINT_WAKEUP: u32 = 1 << 4;
pub const UDINT_EORSM: u32 = 1 << 5;
pub const UDINT_UPRSM: u32 = 1 << 6;

pub struct UdintFlags(u32);

impl fmt::Debug for UdintFlags {
    #[allow(unused_must_use)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let w: u32 = self.0;

        write!(f, "{{");
        if w & UDINT_WAKEUP != 0 {
            write!(f, "w");
        }
        if w & UDINT_SOF != 0 {
            write!(f, "s");
        }

        if w & UDINT_SUSP != 0 {
            write!(f, " SUSP");
        }
        if w & UDINT_EORST != 0 {
            write!(f, " EORST");
        }
        if w & UDINT_EORSM != 0 {
            write!(f, " EORSM");
        }
        if w & UDINT_UPRSM != 0 {
            write!(f, " UPRSM");
        }

        for i in 0..9 {
            if w & (1 << (12 + i)) != 0 {
                write!(f, " EP{}", i);
            }
        }
        write!(f, "}}")
    }
}

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
pub const CTRLDIR: u32 = 1 << 17;
pub const STALLRQ: u32 = 1 << 19;

pub struct UestaFlags(u32);

impl fmt::Debug for UestaFlags {
    #[allow(unused_must_use)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let w: u32 = self.0;

        write!(f, "{{");
        if w & TXIN != 0 {
            write!(f, "TXIN ");
        }
        if w & RXOUT != 0 {
            write!(f, "RXOUT ");
        }
        if w & RXSTP != 0 {
            write!(f, "RXSTP");
        }
        if w & ERRORF != 0 {
            write!(f, "/ERRORF ");
        }
        if w & NAKOUT != 0 {
            write!(f, "NAKOUT ");
        }
        if w & NAKIN != 0 {
            write!(f, "NAKIN ");
        }
        if w & STALLED != 0 {
            write!(f, "STALLED");
        }
        if w & CRCERR != 0 {
            write!(f, "/CRCERR ");
        }
        if w & RAMACERR != 0 {
            write!(f, "RAMACERR ");
        }
        write!(f, "NBUSYBK={} ", (w >> 12) & 0x3);
        write!(f, "CURBK={} ", (w >> 14) & 0x3);
        write!(f, "CTRLDIR={}", if w & CTRLDIR != 0 { "IN" } else { "OUT" });
        write!(f, "}}")
    }
}

/*
pub fn debug_regs() {
    debug!(
        "    registers:\
         \n    USBFSM={:08x}\
         \n    USBCON={:08x}\
         \n    USBSTA={:08x}\
         \n     UDESC={:08x}\
         \n     UDCON={:08x}\
         \n    UDINTE={:08x}\
         \n     UDINT={:08x}\
         \n     UERST={:08x}\
         \n    UECFG0={:08x}\
         \n    UECON0={:08x}",
        USBFSM.read(),
        USBCON.read(),
        USBSTA.read(),
        UDESC.read(),
        UDCON.read(),
        UDINTE.read(),
        UDINT.read(),
        UERST.read(),
        UECFG0.read(),
        UECON0.read()
    );
}
*/
