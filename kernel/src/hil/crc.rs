pub trait CRC {
    fn get_version() -> u32;
    fn compute(&mut self, data: &[u8]) -> bool;
}

pub trait Client {
    fn receive_result(&self, u32);
}
