use crate::{ModuleId, ResResult, Resolver, ROOT_MODULE};
use ast::{Ast, Item, ItemKind, Path, Visitor};
use std::ops::{Deref, DerefMut};

struct ImportResolver<'a, 'r> {
    resolver: &'a mut Resolver<'r>,
}

impl<'a, 'r> ImportResolver<'a, 'r> {
    pub fn new(resolver: &'a mut Resolver<'r>) -> Self {
        Self { resolver }
    }

    fn resolve_path_to_module(&mut self, path: &Path) -> ResResult<ModuleId> {
        todo!()
    }

    fn resolve_use_path(&mut self, path: &Path) {
    }
}

impl<'a> Visitor<'a> for ImportResolver<'a, '_> {
    fn visit_item(&mut self, item: &'a Item) {
        if let ItemKind::Use(path) = &item.kind {
            self.resolve_use_path(path)
        }
    }
}

impl<'r> Resolver<'r> {
    pub fn resolve_imports(&mut self, prog: &Ast) {
        let mut visitor = ImportResolver::new(self);
        visitor.visit_ast(prog);
    }
}

impl<'a, 'r> Deref for ImportResolver<'a, 'r> {
    type Target = Resolver<'r>;

    fn deref(&self) -> &Self::Target {
        &self.resolver
    }
}

impl<'a, 'r> DerefMut for ImportResolver<'a, 'r> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.resolver
    }
}
