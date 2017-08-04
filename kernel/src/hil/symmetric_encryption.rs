//! Interfaces for encryption and decryption using symmetric ciphers

use returncode::ReturnCode;

pub trait Client {
    fn crypt_done(&self, data: &'static mut [u8]);
}

pub const AES128_BLOCK_SIZE: usize = 16;

pub trait AES128Ctr {
    // Request an encryption/decryption.
    // If no buffer is returned, the client's `crypt_done` callback
    // will eventually be invoked with the same buffer that was passed.
    fn crypt(&self,
             client: &'static Client,
             encrypting: bool,
             key: &'static [u8; AES128_BLOCK_SIZE],
             init_ctr: &'static [u8; AES128_BLOCK_SIZE],
             data: &'static mut [u8],
             start_index: usize,
             stop_index: usize) -> (ReturnCode, Option<&'static mut [u8]>);
}

pub trait AES128CBC {
    // Request an encryption/decryption.
    // If no buffer is returned, the client's `crypt_done` callback
    // will eventually be invoked with the same buffer that was passed.
    fn crypt(&self,
             client: &'static Client,
             encrypting: bool,
             key: &'static [u8; AES128_BLOCK_SIZE],
             iv: &'static [u8; AES128_BLOCK_SIZE],
             data: &'static mut [u8],
             start_index: usize,
             stop_index: usize) -> (ReturnCode, Option<&'static mut [u8]>);
}
