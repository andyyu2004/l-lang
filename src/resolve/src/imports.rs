use crate::{ModuleId, ResResult, ResolutionError, Resolver, ROOT_MODULE};
use ast::{Ast, Item, ItemKind, Path, Visitor};
use std::ops::{Deref, DerefMut};

struct ImportResolver<'a, 'r> {
    resolver: &'a mut Resolver<'r>,
}

impl<'a, 'r> ImportResolver<'a, 'r> {
    pub fn new(resolver: &'a mut Resolver<'r>) -> Self {
        Self { resolver }
    }

    /// resolves path to a local module
    fn resolve_local_module(&mut self, path: &Path) -> ResResult<'a, ModuleId> {
        let mut module = ROOT_MODULE;
        for segment in &path.segments {
            debug_assert!(segment.args.is_none());
            match self.find_module(module, segment.ident) {
                Some(m) => module = m,
                None =>
                    return Err(self.build_error(
                        path.span,
                        ResolutionError::UnresolvedModule(segment.clone(), path.clone()),
                    )),
            }
        }
        Ok(module)
    }

    fn resolve_extern_module(&mut self, _path: &Path) -> ResResult<'a, ModuleId> {
        todo!()
    }

    fn resolve_use_path(&mut self, path: &Path) {
        // we use the original error if both local and external resolution fails
        let _module = self
            .resolve_local_module(path)
            .or_else(|err| self.resolve_extern_module(path).map_err(|_| err));
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
