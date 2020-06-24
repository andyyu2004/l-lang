use num_enum::TryFromPrimitive;

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, TryFromPrimitive)]
pub enum Type {
    /// i64
    I,
    /// u64
    U,
    /// f64
    D,
    /// ref
    R,
}

impl Type {
    pub fn size(&self) -> usize {
        match self {
            Self::I | Self::U | Self::D | Self::R => 8,
        }
    }
}
