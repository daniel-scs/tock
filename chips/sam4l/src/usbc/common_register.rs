use core::marker::PhantomData;

/// A memory-mapped register
#[derive(Copy, Clone)]
pub struct Reg<T> {
    addr: *mut u32,
    phantom: PhantomData<*mut T>,
}

impl<T: FromWord + ToWord> Reg<T> {
    pub const fn new(addr: *mut u32) -> Self {
        Reg { addr: addr, phantom: PhantomData }
    }

    #[inline]
    pub fn read(self) -> T {
        unsafe { T::from_word(::core::ptr::read_volatile(self.addr)) }
    }

    #[inline]
    pub fn write(self, val: T) {
        unsafe { ::core::ptr::write_volatile(self.addr, T::to_word(val)); }
    }
}

impl Reg<u32> {
    #[inline]
    pub fn set_bit(self, bit_index: u32) {
        let w = self.read();
        let bit = 1 << bit_index;
        self.write(w | bit);
    }
}

/// A write-only memory-mapped register
pub struct RegW<T> {
    addr: *mut u32,
    phantom: PhantomData<*mut T>,
}

impl<T: FromWord + ToWord> RegW<T> {
    pub const fn new(addr: *mut u32) -> Self {
        RegW { addr: addr, phantom: PhantomData }
    }

    #[inline]
    pub fn write(self, val: T) {
        unsafe { ::core::ptr::write_volatile(self.addr, T::to_word(val)); }
    }
}

/// A read-only memory-mapped register
pub struct RegR<T> {
    addr: *const u32,
    phantom: PhantomData<*mut T>,
}

impl<T: FromWord + ToWord> RegR<T> {
    pub const fn new(addr: *const u32) -> Self {
        RegR { addr: addr, phantom: PhantomData }
    }

    #[inline]
    pub fn read(self) -> T {
        unsafe { T::from_word(::core::ptr::read_volatile(self.addr)) }
    }
}

/// An array of memory-mapped registers
pub struct Regs<T> {
    addr: *mut u32,
    phantom: PhantomData<*mut T>,
}

impl<T: FromWord + ToWord> Regs<T> {
    pub const fn new(addr: *mut u32) -> Self {
        Regs { addr: addr, phantom: PhantomData }
    }

    pub fn n(&self, index: u32) -> Reg<T> {
        unsafe { Reg::new(self.addr.offset(index as isize)) }
    }
}

/// An array of write-only memory-mapped registers
pub struct RegsW<T> {
    addr: *mut u32,
    phantom: PhantomData<*mut T>,
}

impl<T: FromWord + ToWord> RegsW<T> {
    pub const fn new(addr: *mut u32) -> Self {
        RegsW { addr: addr, phantom: PhantomData }
    }

    pub fn n(&self, index: u32) -> RegW<T> {
        unsafe { RegW::new(self.addr.offset(index as isize)) }
    }
}

/// An array of read-only memory-mapped registers
pub struct RegsR<T> {
    addr: *const u32,
    phantom: PhantomData<*const T>,
}

impl<T: FromWord + ToWord> RegsR<T> {
    pub const fn new(addr: *const u32) -> Self {
        RegsR { addr: addr, phantom: PhantomData }
    }

    pub fn n(&self, index: u32) -> RegR<T> {
        unsafe { RegR::new(self.addr.offset(index as isize)) }
    }
}

/// A bitfield of a memory-mapped register
pub struct BitField<T> {
    reg: Reg<u32>,
    shift: u32,
    bits: u32,
    phantom: PhantomData<*mut T>,
}

impl<T: ToWord> BitField<T> {
    pub const fn new(reg: Reg<u32>, shift: u32, bits: u32) -> Self {
        BitField { reg: reg, shift: shift, bits: bits, phantom: PhantomData }
    }

    #[inline]
    pub fn write(self, val: T) {
        let w = self.reg.read();
        let mask = self.bits << self.shift;
        let val_bits = (val.to_word() & self.bits) << self.shift;
        self.reg.write(w & !mask | val_bits);
    }
}

pub trait ToWord {
    fn to_word(self) -> u32;
}

impl ToWord for u32 {
    #[inline]
    fn to_word(self) -> u32 { self }
}

impl ToWord for bool {
    #[inline]
    fn to_word(self) -> u32 { if self { 1 } else { 0 } }
}

pub trait FromWord {
    fn from_word(u32) -> Self;
}

impl FromWord for u32 {
    #[inline]
    fn from_word(w: u32) -> Self { w }
}
