#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Symbol(pub usize);

impl Symbol {
    pub const fn new(n: usize) -> Self {
        Self(n)
    }
}
