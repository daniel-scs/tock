// Generic interface for CRC computation

use returncode::ReturnCode;

pub trait CRC {
    // Call this method exactly once before any other calls
    fn init(&self) -> ReturnCode;

    fn get_version(&self) -> u32;

    // Initiate a CRC calculation
    fn compute(&self, data: &[u8]) -> ReturnCode;
}

pub trait Client {
    // Receive the successful result of a CRC calculation
    fn receive_result(&self, u32);
}
