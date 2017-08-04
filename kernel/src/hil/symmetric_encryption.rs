//! Interfaces for accessing encryption and decryption of symmetric ciphers.

pub trait Client<'a> {
    fn crypt_done(&self, data: &'a mut [u8]);
}

pub const BLOCK_SIZE: usize = 16;

pub trait AES128Ctr {
    type Request;

    // Create a request structure
    fn create_request(client: &Client,
                      encrypting: bool,
                      key: &[u8; BLOCK_SIZE],
                      init_ctr: &[u8; BLOCK_SIZE],
                      data: &mut [u8],
                      start_index: usize,
                      stop_index: usize) -> Option<Self::Request>;

    // Request an encryption/decryption.
    // Returns true if the request is valid and the client will
    // eventually receive a callback.
    fn crypt(&self, request: &mut Self::Request) -> bool;
}

pub trait AES128CBC {
    type Request;

    // Create a request structure
    fn create_request(client: &Client,
                      encrypting: bool,
                      key: &[u8; BLOCK_SIZE],
                      iv: &[u8; BLOCK_SIZE],
                      data: &mut [u8],
                      start_index: usize,
                      stop_index: usize) -> Option<Self::Request>;

    // Request an encryption/decryption.
    // Returns true if the request is valid and the client will
    // eventually receive a callback.
    fn crypt(&self, request: &mut Self::Request) -> bool;
}
