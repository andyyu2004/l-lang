use super::{LateResolver, NS};
use crate::ast::{self, Ident, Pattern, PatternKind, Visitor};
use crate::error::ResolutionError;
use crate::ir::Res;
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};

pub struct PatternResolutionCtx<'a, 'b, 'r, 'ast> {
    /// to keep track of names used in pattern already
    names: HashSet<Ident>,
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
                    self.emit_error(
                        vec![prev.span, ident.span],
                        ResolutionError::DuplicatePatternIdentifier(ident),
                    );
                }
                self.names.insert(ident);
                self.def_val(ident, Res::Local(pat.id));
            }
            PatternKind::Path(path) | PatternKind::Variant(path, _) =>
                self.resolve_path(path, NS::Value),
            PatternKind::Wildcard | PatternKind::Tuple(..) | PatternKind::Paren(..) => {}
        }
        ast::walk_pat(self, pat);
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
