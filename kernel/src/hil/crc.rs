pub trait CRC {
    fn get_version() -> u32;
    fn compute(data: &[u8]) -> bool;
}

pub trait Client {
}
