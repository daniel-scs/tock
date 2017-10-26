use capsules::test::aes::{Test};
use sam4l::aes::{Aes};
use kernel::hil::symmetric_encryption::AES128_BLOCK_SIZE;

pub fn static_init_test(aes: &'static mut Aes<'static>) -> &'static mut Test<'static, Aes<'static>> {
    unsafe {
        let source = static_init!([u8; 4 * AES128_BLOCK_SIZE], [0; 4 * AES128_BLOCK_SIZE]);
        let data = static_init!([u8; 6 * AES128_BLOCK_SIZE], [0; 6 * AES128_BLOCK_SIZE]);
        let key = static_init!([u8; AES128_BLOCK_SIZE], [0; AES128_BLOCK_SIZE]);
        let iv = static_init!([u8; AES128_BLOCK_SIZE], [0; AES128_BLOCK_SIZE]);

        static_init!(Test<'static, Aes>, Test::new(aes, key, iv, source, data))
    }
}
