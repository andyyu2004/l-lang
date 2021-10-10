use crate::{LateResolver, ResolutionError, NS};
use lc_ast::{self, Ident, Pattern, PatternKind, Visitor};
use ir::Res;
use rustc_hash::FxHashSet;
use std::ops::{Deref, DerefMut};

pub struct PatternResolutionCtx<'a, 'b, 'r, 'ast> {
    /// to keep track of names used in pattern already
    names: FxHashSet<Ident>,
    resolver: &'a mut LateResolver<'b, 'r, 'ast>,
}

impl<'a, 'b, 'r, 'ast> PatternResolutionCtx<'a, 'b, 'r, 'ast> {
    pub fn new(resolver: &'a mut LateResolver<'b, 'r, 'ast>) -> Self {
        Self { resolver, names: Default::default() }
    }

    pub fn resolve_pattern(&mut self, pat: &'ast Pattern) {
        match &pat.kind {
            &PatternKind::Ident(ident, ..) => {
                if let Some(prev) = self.names.get(&ident) {
                    self.build_error(
                        ident.span,
                        ResolutionError::DuplicatePatternIdentifier(ident),
                    )
                    .span(prev.span)
                    .emit();
                }
                self.names.insert(ident);
                self.def_val(ident, Res::Local(pat.id));
            }
            PatternKind::Struct(path, ..)
            | PatternKind::Path(path)
            | PatternKind::Variant(path, _) => self.resolve_path(path, NS::Value),
            PatternKind::Lit(..)
            | PatternKind::Box(..)
            | PatternKind::Tuple(..)
            | PatternKind::Paren(..)
            | PatternKind::Wildcard => {}
        }
        lc_ast::walk_pat(self, pat);
    }
}

impl<'a, 'b, 'r, 'ast> Visitor<'ast> for PatternResolutionCtx<'a, 'b, 'r, 'ast> {
    fn visit_pattern(&mut self, pat: &'ast Pattern) {
        self.resolve_pattern(pat)
    }
}

impl<'a, 'b, 'r, 'ast> Deref for PatternResolutionCtx<'a, 'b, 'r, 'ast> {
    type Target = LateResolver<'b, 'r, 'ast>;

    fn deref(&self) -> &Self::Target {
        &self.resolver
    }
}

impl<'a, 'b, 'r, 'ast> DerefMut for PatternResolutionCtx<'a, 'b, 'r, 'ast> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.resolver
    }
}
