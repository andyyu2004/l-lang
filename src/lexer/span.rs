#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Span {
    /// lo is inclusive
    pub lo: usize,
    /// lo is exclusive
    pub hi: usize,
}

impl Span {
    pub const fn new(lo: usize, hi: usize) -> Self {
        Self { lo, hi }
    }

    pub fn merge(&self, with: &Span) -> Self {
        Self {
            lo: self.lo,
            hi: with.hi,
        }
    }
}
