//! Interfaces for accessing encryption and decryption of symmetric ciphers.

pub const BLOCK_SIZE: usize = 16;

pub trait Client<'a> {
    fn crypt_done(&self, data: &'a mut [u8]);
}

pub trait AES128Ctr {
    // Request an encryption/decryption.
    // Returns true if the request is valid and the client will
    // eventually receive a callback.
    fn crypt(&self,
             client: &Client,
             encrypting: bool,
             key: &[u8; BLOCK_SIZE],
             init_ctr: &[u8; BLOCK_SIZE],
             data: &mut [u8],
             start_index: usize,
             stop_index: usize) -> bool;
}

pub trait AES128CBC {
    // Request an encryption/decryption.
    // Returns true if the request is valid and the client will
    // eventually receive a callback.
    fn crypt(&self,
             client: &Client,
             encrypting: bool,
             key: &[u8; BLOCK_SIZE],
             iv: &[u8; BLOCK_SIZE],
             data: &mut [u8],
             start_index: usize,
             stop_index: usize) -> bool;
}
