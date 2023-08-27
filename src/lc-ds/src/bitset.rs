use lc_index::Idx;
use std::marker::PhantomData;

type Word = u8;
const WORD_BYTES: usize = std::mem::size_of::<Word>();
const WORD_BITS: usize = WORD_BYTES * 8;

// we make it generic over T so the iterator knows what type to return
#[derive(Debug, Clone)]
pub struct Bitset<T: Idx> {
    words: Vec<Word>,
    pd: std::marker::PhantomData<T>,
}

impl<T: Idx> Bitset<T> {
    pub fn new(size: usize) -> Self {
        Self { words: vec![0; Self::words_required(size)], pd: std::marker::PhantomData }
    }

    /// return whether the set has changed
    pub fn insert(&mut self, idx: T) -> bool {
        let has_changed = !self.is_set(idx);
        self.set(idx);
        has_changed
    }

    pub fn set(&mut self, idx: T) {
        let (index, shift) = Self::bit_indices(idx);
        self.words[index] |= shift;
    }

    pub fn is_unset(&self, idx: T) -> bool {
        !self.is_set(idx)
    }

    pub fn is_set(&self, idx: T) -> bool {
        let (index, shift) = Self::bit_indices(idx);
        self.words[index] & shift != 0
    }

    pub fn unset(&mut self, idx: T) {
        let (index, shift) = Self::bit_indices(idx);
        self.words[index] &= !shift;
    }

    fn bit_indices(idx: impl Idx) -> (usize, Word) {
        let idx = idx.index();
        let index = idx / WORD_BITS;
        let shift = 1 << (idx % WORD_BITS);
        (index, shift)
    }

    /// The numbers of words required to store `size` bits
    /// ```
    /// assert_eq!(lc_ds::Bitset::<usize>::words_required(0), 0);
    /// for i in 1..8 {
    ///     assert_eq!(lc_ds::Bitset::<usize>::words_required(i), 1);
    /// }
    ///
    /// for i in 9..16 {
    ///     assert_eq!(lc_ds::Bitset::<usize>::words_required(i), 2);
    /// }
    /// ```
    pub fn words_required(size: usize) -> usize {
        (size + WORD_BITS - 1) / WORD_BITS
    }

    pub fn iter(&self) -> BitsetIter<'_, T> {
        BitsetIter::new(&self.words)
    }
}

pub struct BitsetIter<'a, T: Idx> {
    words: std::slice::Iter<'a, Word>,
    // copy of the current word that we mutate
    word: Word,
    count: usize,
    pd: PhantomData<T>,
}

impl<'a, T: Idx> BitsetIter<'a, T> {
    fn new(words: &'a [Word]) -> Self {
        Self { words: words.iter(), count: 0, word: 0, pd: PhantomData }
    }
}

impl<'a, T: Idx> Iterator for BitsetIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        while self.word == 0 {
            self.word = *self.words.next()?;
            self.count += 1;
        }

        let bit_idx = self.word.trailing_zeros() as usize;
        // unset the bit we return so we don't return it again
        self.word ^= 1 << bit_idx;
        Some(T::new(bit_idx + (self.count - 1) * WORD_BITS))
    }
}

impl<'a, T: Idx> IntoIterator for &'a Bitset<T> {
    type IntoIter = BitsetIter<'a, T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitset_iter() {
        let mut bitset = Bitset::new(8);
        bitset.set(1);
        bitset.set(3);
        bitset.set(4);
        bitset.set(6);
        for i in bitset.iter() {
            dbg!(i);
        }
        assert_eq!(vec![1, 3, 4, 6], bitset.iter().collect::<Vec<_>>());
    }

    #[test]
    fn test_bitset() {
        let mut bitset = Bitset::new(8);
        assert_eq!(bitset.words.len(), 1);
        bitset.set(5);
        // should be 0010 0000
        assert_eq!(bitset.words[0], 32);

        // 0010 1000
        bitset.set(3);
        assert_eq!(bitset.words[0], 40);
        assert!(bitset.is_set(3));

        // 0010 1001
        bitset.set(0);
        assert!(bitset.is_set(0));
        assert_eq!(bitset.words[0], 41);

        bitset.unset(3);
        assert_eq!(bitset.words[0], 33);
        assert!(!bitset.is_set(3));
    }
}
