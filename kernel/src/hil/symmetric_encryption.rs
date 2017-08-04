//! Interfaces for accessing encryption and decryption of symmetric ciphers.

pub trait Client {
    fn crypt_done(&self, data: &'static mut [u8]);
}

pub const BLOCK_SIZE: usize = 16;

pub trait AES128Ctr {
    // Request an encryption/decryption.
    // Returns true if the request is valid and the client will
    // eventually receive a callback.
    fn crypt(&self,
             client: &'static Client,
             encrypting: bool,
             key: &'static [u8; BLOCK_SIZE],
             init_ctr: &'static [u8; BLOCK_SIZE],
             data: &'static mut [u8],
             start_index: usize,
             stop_index: usize) -> bool;
}

pub trait AES128CBC {
    // Request an encryption/decryption.
    // Returns true if the request is valid and the client will
    // eventually receive a callback.
    fn crypt(&self,
             client: &'static Client,
             encrypting: bool,
             key: &'static [u8; BLOCK_SIZE],
             iv: &'static [u8; BLOCK_SIZE],
             data: &'static mut [u8],
             start_index: usize,
             stop_index: usize) -> bool;
}
