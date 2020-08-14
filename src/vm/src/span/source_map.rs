use crate::span::Span;
use std::ops::{self, Index};

#[derive(Debug)]
pub struct SourceMap {
    pub files: Vec<SourceFile>,
}

#[derive(Debug)]
pub struct SourceFile {
    pub src: String,
}

impl SourceFile {
    pub fn new(src: &str) -> Self {
        Self { src: src.to_owned() }
    }
}

impl SourceMap {
    // tmp function to use for now
    pub fn main_file(&self) -> &SourceFile {
        &self.files[0]
    }

    // just one sourcefile for now
    pub fn new(src: &str) -> Self {
        Self { files: vec![SourceFile::new(src)] }
    }

    pub fn span_to_string(&self, span: Span) -> String {
        self.files[0].src[span.lo..span.hi].to_owned()
    }
}

impl<'a> Index<Span> for &'a SourceFile {
    type Output = str;

    fn index(&self, index: Span) -> &Self::Output {
        let Span { lo, hi } = index;
        &self.src[lo..hi]
    }
}
