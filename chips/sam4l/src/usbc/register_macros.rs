//! Macros for defining USBC registers

#[macro_export]
macro_rules! reg {
    [ $offset:expr, $description:expr, $name:ident, $ty:ident, "RW" ] => {

        #[allow(non_snake_case)]
        mod $name {
            use kernel::common::volatile_cell::VolatileCell;
            pub struct $name(pub VolatileCell<u32>);
        }

        impl RegisterRW for $name::$name {
            #[inline]
            fn read(&self) -> u32 {
                self.0.get()
            }

            #[inline]
            fn write(&self, val: u32) {
                self.0.set(val);
            }
        }

        pub const $name: StaticRef<$name::$name> = unsafe {
            StaticRef::new((USBC_BASE + $offset) as *const $name::$name)
        };
    };

    [ $offset:expr, $description:expr, $name:ident, $ty:ident, "R" ] => {

        #[allow(non_snake_case)]
        mod $name {
            use kernel::common::volatile_cell::VolatileCell;
            pub struct $name(pub VolatileCell<u32>);
        }

        impl RegisterR for $name::$name {
            #[inline]
            fn read(&self) -> u32 {
                self.0.get()
            }
        }

        pub const $name: StaticRef<$name::$name> = unsafe {
            StaticRef::new((USBC_BASE + $offset) as *const $name::$name)
        };
    };

    [ $offset:expr, $description:expr, $name:ident, $ty: ident, "W" ] => {

        #[allow(non_snake_case)]
        mod $name {
            use kernel::common::volatile_cell::VolatileCell;
            pub struct $name(pub VolatileCell<u32>);
        }

        impl RegisterW for $name::$name {
            #[inline]
            fn write(&self, val: u32) {
                self.0.set(val);
            }
        }

        pub const $name: StaticRef<$name::$name> = unsafe {
            StaticRef::new((USBC_BASE + $offset) as *const $name::$name)
        };
    };
}

#[macro_export]
macro_rules! regs {
    [ $offset:expr, $description:expr, $name:ident, $ty:ident, "RW", $count:expr ] => {

        #[allow(non_snake_case)]
        mod $name {
            use kernel::common::volatile_cell::VolatileCell;
            pub struct $name(pub VolatileCell<u32>);
        }

        impl RegisterRW for $name::$name {
            #[inline]
            fn read(&self) -> u32 {
                self.0.get()
            }

            #[inline]
            fn write(&self, val: u32) {
                self.0.set(val);
            }
        }

        pub const $name: StaticRef<[$name::$name; $count]> = unsafe {
            StaticRef::new((USBC_BASE + $offset) as *const [$name::$name; $count])
        };
    };

    [ $offset:expr, $description:expr, $name:ident, $ty:ident, "R", $count:expr ] => {

        #[allow(non_snake_case)]
        mod $name {
            use kernel::common::volatile_cell::VolatileCell;
            pub struct $name(pub VolatileCell<u32>);
        }

        impl RegisterR for $name::$name {
            #[inline]
            fn read(&self) -> u32 {
                self.0.get()
            }
        }

        pub const $name: StaticRef<[$name::$name; $count]> = unsafe {
            StaticRef::new((USBC_BASE + $offset) as *const [$name::$name; $count])
        };
    };

    [ $offset:expr, $description:expr, $name:ident, $ty:ident, "W", $count:expr ] => {

        #[allow(non_snake_case)]
        mod $name {
            use kernel::common::volatile_cell::VolatileCell;
            pub struct $name(pub VolatileCell<u32>);
        }

        impl RegisterW for $name::$name {
            #[inline]
            fn write(&self, val: u32) {
                self.0.set(val);
            }
        }

        pub const $name: StaticRef<[$name::$name; $count]> = unsafe {
            StaticRef::new((USBC_BASE + $offset) as *const [$name::$name; $count])
        };
    };
}

#[macro_export]
macro_rules! bitfield {
    [ $reg:ident, $field:ident, "RW", $valty:ty, $shift:expr, $src_mask:expr ] => {

        #[allow(non_snake_case)]
        mod $field {
            pub struct $field;
        }

        impl $field::$field {
            pub fn write(self, val: $valty) {
                let w = $reg.read();
                let dst_mask = $src_mask << $shift;
                let val_bits = (val.to_word() & $src_mask) << $shift;
                $reg.write((w & !dst_mask) | val_bits);
            }
        }

        pub const $field: $field::$field = $field::$field;
    };

    /*
    [ $reg:ident, $field:ident, "W", $t:ty, $shift:expr, $bits:expr ] => {
        #[allow(dead_code)]
        pub const $field: BitFieldW<$t> = BitFieldW::new($reg, $shift, $bits);
    };

    [ $reg:ident, $field:ident, "R", $t:ty, $shift:expr, $bits:expr ] => {
        #[allow(dead_code)]
        pub const $field: BitFieldR<$t> = BitFieldR::new($reg, $shift, $bits);
    };
    */
}
