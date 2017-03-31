#![allow(non_upper_case_globals)]

// A memory-mapped register
pub struct Reg(*mut u32);

impl Reg {
    pub fn read(self) -> u32 {
        unsafe { ::core::ptr::read_volatile(self.0) }
    }

    pub fn write(self, n: u32) {
        unsafe { ::core::ptr::write_volatile(self.0, n); }
    }
}

// A write-only memory-mapped register
pub struct RegW(*mut u32);

impl RegW {
    pub fn write(self, n: u32) {
        unsafe { ::core::ptr::write_volatile(self.0, n); }
    }
}

// A read-only memory-mapped register
pub struct RegR(*const u32);

impl RegR {
    pub fn read(self) -> u32 {
        unsafe { ::core::ptr::read_volatile(self.0) }
    }
}

// An array of memory-mapped registers
pub struct Regs(*mut u32);

impl Regs {
    pub fn n(&self, index: u32) -> Reg {
        unsafe { Reg(self.0.offset(index as isize)) }
    }
}

// An array of write-only memory-mapped registers
pub struct RegsW(*mut u32);

impl RegsW {
    pub fn n(&self, index: u32) -> RegW {
        unsafe { RegW(self.0.offset(index as isize)) }
    }
}

// An array of read-only memory-mapped registers
pub struct RegsR(*mut u32);

impl RegsR {
    pub fn n(&self, index: u32) -> RegR {
        unsafe { RegR(self.0.offset(index as isize)) }
    }
}

// Base address of USBC registers.  See "7.1 Product Mapping"
const USBC_BASE: u32 = 0x400A5000;

macro_rules! reg {
    [ $offset:expr, $description:expr, $name:ident, "RW" ] => {
        #[allow(dead_code)]
        const $name: Reg = Reg((USBC_BASE + $offset) as *mut u32);
    };

    [ $offset:expr, $description:expr, $name:ident, "R" ] => {
        #[allow(dead_code)]
        const $name: RegR = RegR((USBC_BASE + $offset) as *mut u32);
    };

    [ $offset:expr, $description:expr, $name:ident, "W" ] => {
        #[allow(dead_code)]
        const $name: RegW = RegW((USBC_BASE + $offset) as *mut u32);
    };
}

macro_rules! regs {
    [ $offset:expr, $description:expr, $name:ident, "RW", $count:expr ] => {
        #[allow(dead_code)]
        const $name: Regs = Regs((USBC_BASE + $offset) as *mut u32);
    };

    [ $offset:expr, $description:expr, $name:ident, "R", $count:expr ] => {
        #[allow(dead_code)]
        const $name: RegsR = RegsR((USBC_BASE + $offset) as *mut u32);
    };

    [ $offset:expr, $description:expr, $name:ident, "W", $count:expr ] => {
        #[allow(dead_code)]
        const $name: RegsW = RegsW((USBC_BASE + $offset) as *mut u32);
    };
}

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

reg![0x0800, "General Control Register", USBCON, "RW"];
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
