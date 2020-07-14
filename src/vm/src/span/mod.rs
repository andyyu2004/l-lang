mod ctx;
mod source_map;

crate use ctx::Ctx;
crate use source_map::SourceMap;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Span {
    /// lo is inclusive
    pub lo: usize,
    /// lo is exclusive
    pub hi: usize,
}

impl Span {
    pub const fn new(lo: usize, hi: usize) -> Self {
        assert!(lo <= hi);
        Self { lo, hi }
    }

    pub fn merge(&self, with: &Span) -> Self {
        Self { lo: self.lo, hi: with.hi }
    }
}
