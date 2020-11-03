use crate::Span;
use std::ops::Index;

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

    pub fn span_to_slice(&self, span: Span) -> &str {
        &self.files[0].src[span.start().to_usize()..span.end().to_usize()]
    }

    pub fn span_to_string(&self, span: Span) -> String {
        self.span_to_slice(span).to_owned()
    }
}

impl<'a> Index<Span> for &'a SourceFile {
    type Output = str;

    fn index(&self, span: Span) -> &Self::Output {
        &self.src[span.start().to_usize()..span.end().to_usize()]
    }
}
