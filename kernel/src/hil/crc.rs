// Generic interface for CRC computation

use returncode::ReturnCode;

pub enum Polynomial {
	CCIT8023,   // Polynomial 0x04C11DB7
	CASTAGNOLI, // Polynomial 0x1EDC6F41
	CCIT16,		// Polynomial 0x1021
}

pub fn poly_from_int(i: usize) -> Option<Polynomial> {
    match i {
        0 => Some(Polynomial::CCIT8023),
        1 => Some(Polynomial::CASTAGNOLI),
        2 => Some(Polynomial::CCIT16),
        _ => None
    }
}

pub trait CRC {
    // Call this method exactly once before any other calls
    fn init(&self) -> ReturnCode;

    fn get_version(&self) -> u32;

    // Initiate a CRC calculation
    fn compute(&self, data: &[u8], Polynomial) -> ReturnCode;
}

pub trait Client {
    // Receive the successful result of a CRC calculation
    fn receive_result(&self, u32);
}
