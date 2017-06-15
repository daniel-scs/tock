//! Macros for defining USBC registers

#[macro_export]
macro_rules! reg {
    [ $offset:expr, $description:expr, $name:ident, "RW" ] => {

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

    [ $offset:expr, $description:expr, $name:ident, "R" ] => {

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

    [ $offset:expr, $description:expr, $name:ident, "W" ] => {

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
    [ $offset:expr, $description:expr, $name:ident, "RW", $count:expr ] => {

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

    [ $offset:expr, $description:expr, $name:ident, "R", $count:expr ] => {

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

    [ $offset:expr, $description:expr, $name:ident, "W", $count:expr ] => {

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
macro_rules! registers {
    [ $base:expr, {
        $( $offset:expr => { $description:expr, $name:ident, $access:tt } ),*
    } ] => {
        $( reg![ $offset, $description, $name, $access ]; )*
    };
}

#[macro_export]
macro_rules! bitfield {
    [ $reg:ident, $field:ident, "RW", $valty:ty, $shift:expr, $mask:expr ] => {

        #[allow(non_snake_case)]
        mod $field {
            pub struct $field;
        }

        impl $field::$field {
            pub fn write(self, val: $valty) {
                let w = $reg.read();
                let val_bits = (val.to_word() & $mask) << $shift;
                $reg.write((w & !($mask << $shift)) | val_bits);
            }
        }

        pub const $field: $field::$field = $field::$field;
    };

    [ $reg:ident, $field:ident, "R", $valty:ty, $shift:expr, $mask:expr ] => {

        #[allow(non_snake_case)]
        mod $field {
            pub struct $field;
        }

        impl $field::$field {
            pub fn read(self) -> $valty {
                FromWord::from_word(($reg.read() >> $shift) & $mask)
            }
        }

        pub const $field: $field::$field = $field::$field;
    };
}
