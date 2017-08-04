//! Interfaces for accessing encryption and decryption of symmetric ciphers.

pub trait Client<'a> {
    fn crypt_done(&self, data: &'a mut [u8]);
}

pub const BLOCK_SIZE: usize = 16;

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum ConfidentialityMode {
    ECB = 0,
    CBC,
    CFB,
    OFB,
    Ctr,
}

// A structure to represent a particular encryption request
pub struct Request<'a> {
    pub client: &'a Client<'a>,

    pub mode: ConfidentialityMode,
    pub encrypting: bool,
    pub key: &'a [u8; BLOCK_SIZE],
    pub iv: &'a [u8; BLOCK_SIZE],
    pub data: &'a mut [u8],

    // The index of the first byte in `data` to encrypt
    pub start_index: usize,
  
    // The index just after the last byte to encrypt
    pub stop_index: usize,
}

impl<'a> Request<'a> {
    // Create a request structure, or None if the arguments are invalid
    pub fn new(client: &'a Client<'a>,
               mode: ConfidentialityMode,
               encrypting: bool,
               key: &'a [u8; BLOCK_SIZE],
               iv: &'a [u8; BLOCK_SIZE],
               data: &'a mut [u8],
               start_index: usize,
               stop_index: usize) -> Option<Request<'a>>
    {
        let len = data.len();
        if len % BLOCK_SIZE != 0
            || start_index > len
            || stop_index > len
            || start_index > stop_index {
            None
        } else {
            Some(Request {
                client: client,
                mode: mode,
                encrypting: encrypting,
                key: key,
                iv: iv,
                data: data,
                start_index: start_index,
                stop_index: stop_index,
            })
        }
    }
}

pub trait AES128Ctr<'a> {
    // Create a request structure
    fn create_request(client: &'a Client<'a>,
                      encrypting: bool,
                      key: &'a [u8; BLOCK_SIZE],
                      init_ctr: &'a [u8; BLOCK_SIZE],
                      data: &'a mut [u8],
                      start_index: usize,
                      stop_index: usize) -> Option<Request<'a>>;

    // Request an encryption/decryption.
    // Returns true if the request is valid and the client will
    // eventually receive a callback.
    fn crypt(&'a self, request: &mut Request<'a>) -> bool;
}

pub trait AES128CBC<'a> {
    // Create a request structure
    fn create_request(client: &'a Client<'a>,
                      encrypting: bool,
                      key: &'a [u8; BLOCK_SIZE],
                      iv: &'a [u8; BLOCK_SIZE],
                      data: &'a mut [u8],
                      start_index: usize,
                      stop_index: usize) -> Option<Request<'a>>;

    // Request an encryption/decryption.
    // Returns true if the request is valid and the client will
    // eventually receive a callback.
    fn crypt(&'a self, request: &mut Request<'a>) -> bool;
}
