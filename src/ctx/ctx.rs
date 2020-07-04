use super::{source_map::SourceFile, SourceMap};
use crate::{arena::DroplessArena, lexer::symbol};

crate struct Ctx {
    pub symbol_interner: symbol::Interner,
    pub source_map: SourceMap,
    pub arena: DroplessArena,
}

impl Ctx {
    // tmp function to use for now
    pub fn main_file(&self) -> &SourceFile {
        &self.source_map.files[0]
    }

    pub fn new(src: &str) -> Self {
        Self {
            symbol_interner: Default::default(),
            source_map: SourceMap::new(src),
            arena: Default::default(),
        }
    }
}
