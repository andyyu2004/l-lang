use crate::span::Span;
use std::ops::Index;

pub struct SourceMap {
    pub files: Vec<SourceFile>,
}

pub struct SourceFile {
    pub src: String,
}

impl SourceFile {
    pub fn new(src: &str) -> Self {
        Self {
            src: src.to_owned(),
        }
    }
}

impl SourceMap {
    // just one sourcefile for now
    pub fn new(src: &str) -> Self {
        Self {
            files: vec![SourceFile::new(src)],
        }
    }
}

impl<'a> Index<Span> for &'a SourceFile {
    type Output = str;
    fn index(&self, index: Span) -> &Self::Output {
        let Span { lo, hi } = index;
        &self.src[lo..hi]
    }
}
