//! Macros for defining USBC registers

#[macro_export]
macro_rules! reg {
    [ $offset:expr, $description:expr, $name:ident, $ty:ident, "RW" ] => {
        pub struct $ty(VolatileCell<u32>);

        impl RegisterRW for $ty {
            #[inline]
            fn read(&self) -> u32 {
                self.0.get()
            }

            #[inline]
            fn write(&self, val: u32) {
                self.0.set(val);
            }
        }

        pub const $name: StaticRef<$ty> = unsafe {
            StaticRef::new((USBC_BASE + $offset) as *const $ty)
        };
    };

    [ $offset:expr, $description:expr, $name:ident, $ty:ident, "R" ] => {
        pub struct $ty(VolatileCell<u32>);

        impl RegisterR for $ty {
            #[inline]
            fn read(&self) -> u32 {
                self.0.get()
            }
        }

        pub const $name: StaticRef<$ty> = unsafe {
            StaticRef::new((USBC_BASE + $offset) as *const $ty)
        };
    };

    [ $offset:expr, $description:expr, $name:ident, $ty: ident, "W" ] => {
        pub struct $ty(VolatileCell<u32>);

        impl RegisterW for $ty {
            #[inline]
            fn write(&self, val: u32) {
                self.0.set(val);
            }
        }

        pub const $name: StaticRef<$ty> = unsafe {
            StaticRef::new((USBC_BASE + $offset) as *const $ty)
        };
    };
}

#[macro_export]
macro_rules! regs {
    [ $offset:expr, $description:expr, $name:ident, $ty:ident, "RW", $count:expr ] => {
        pub struct $ty(VolatileCell<u32>);

        impl RegisterRW for $ty {
            #[inline]
            fn read(&self) -> u32 {
                self.0.get()
            }

            #[inline]
            fn write(&self, val: u32) {
                self.0.set(val);
            }
        }

        pub const $name: StaticRef<[$ty; $count]> = unsafe {
            StaticRef::new((USBC_BASE + $offset) as *const [$ty; $count])
        };
    };

    [ $offset:expr, $description:expr, $name:ident, $ty:ident, "R", $count:expr ] => {
        pub struct $ty(VolatileCell<u32>);

        impl RegisterR for $ty {
            #[inline]
            fn read(&self) -> u32 {
                self.0.get()
            }
        }

        pub const $name: StaticRef<[$ty; $count]> = unsafe {
            StaticRef::new((USBC_BASE + $offset) as *const [$ty; $count])
        };
    };

    [ $offset:expr, $description:expr, $name:ident, $ty:ident, "W", $count:expr ] => {
        pub struct $ty(VolatileCell<u32>);

        impl RegisterW for $ty {
            #[inline]
            fn write(&self, val: u32) {
                self.0.set(val);
            }
        }

        pub const $name: StaticRef<[$ty; $count]> = unsafe {
            StaticRef::new((USBC_BASE + $offset) as *const [$ty; $count])
        };
    };
}

/*
#[macro_export]
macro_rules! bitfield {
    [ $reg:ident, $field:ident, "RW", $t:ty, $shift:expr, $bits:expr ] => {
        #[allow(dead_code)]
        pub const $field: BitField<$t> = BitField::new($reg, $shift, $bits);
    };

    [ $reg:ident, $field:ident, "W", $t:ty, $shift:expr, $bits:expr ] => {
        #[allow(dead_code)]
        pub const $field: BitFieldW<$t> = BitFieldW::new($reg, $shift, $bits);
    };

    [ $reg:ident, $field:ident, "R", $t:ty, $shift:expr, $bits:expr ] => {
        #[allow(dead_code)]
        pub const $field: BitFieldR<$t> = BitFieldR::new($reg, $shift, $bits);
    };
}
*/
