pub trait As8Bytes {
    fn as_bytes(&self) -> [u8; 8];
}

impl As8Bytes for i64 {
    fn as_bytes(&self) -> [u8; 8] {
        self.to_le_bytes()
    }
}

impl As8Bytes for u64 {
    fn as_bytes(&self) -> [u8; 8] {
        self.to_le_bytes()
    }
}

impl As8Bytes for f64 {
    fn as_bytes(&self) -> [u8; 8] {
        self.to_bits().as_bytes()
    }
}
