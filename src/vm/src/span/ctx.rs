use super::{source_map::SourceFile, SourceMap};
use crate::{arena::DroplessArena, lexer::symbol};

pub struct Ctx {
    pub source_map: SourceMap,
    pub arena: DroplessArena,
}

impl Ctx {
    // tmp function to use for now
    pub fn main_file(&self) -> &SourceFile {
        &self.source_map.files[0]
    }

    pub fn new(src: &str) -> Self {
        Self { source_map: SourceMap::new(src), arena: Default::default() }
    }
}
