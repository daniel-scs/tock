//! Macros for defining USBC registers

#[macro_export]
macro_rules! reg {
    [ $offset:expr, $description:expr, $name:ident, "RW" ] => {
        #[allow(dead_code)]
        pub const $name: StaticRef<Register> = unsafe {
            StaticRef::new((USBC_BASE + $offset) as *const Register)
        };
    };

    [ $offset:expr, $description:expr, $name:ident, "R" ] => {
        #[allow(dead_code)]
        pub const $name: StaticRef<RegisterR> = unsafe {
            StaticRef::new((USBC_BASE + $offset) as *const RegisterR)
        };
    };

    [ $offset:expr, $description:expr, $name:ident, "W" ] => {
        #[allow(dead_code)]
        pub const $name: StaticRef<RegisterW> = unsafe {
            StaticRef::new((USBC_BASE + $offset) as *const RegisterW)
        };
    };
}

#[macro_export]
macro_rules! regs {
    [ $offset:expr, $description:expr, $name:ident, "RW", $count:expr ] => {
        #[allow(dead_code)]
        pub const $name: StaticRef<[Register; $count]> = unsafe {
            StaticRef::new((USBC_BASE + $offset) as *const [Register; $count])
        };
    };

    [ $offset:expr, $description:expr, $name:ident, "R", $count:expr ] => {
        #[allow(dead_code)]
        pub const $name: StaticRef<[RegisterR; $count]> = unsafe {
            StaticRef::new((USBC_BASE + $offset) as *const [RegisterR; $count])
        };
    };

    [ $offset:expr, $description:expr, $name:ident, "W", $count:expr ] => {
        #[allow(dead_code)]
        pub const $name: StaticRef<[RegisterW; $count]> = unsafe {
            StaticRef::new((USBC_BASE + $offset) as *const [RegisterW; $count])
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
