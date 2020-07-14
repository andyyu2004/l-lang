use super::AstLoweringCtx;
use crate::ast::*;
use crate::ir;
use itertools::Itertools;

impl<'ir> AstLoweringCtx<'_, 'ir> {
    crate fn lower_pattern(&mut self, pat: &Pattern) -> &'ir ir::Pattern<'ir> {
        self.arena.alloc(self.lower_pattern_inner(pat))
    }

    fn lower_pattern_inner(&mut self, pat: &Pattern) -> ir::Pattern<'ir> {
        let &Pattern { id, span, ref kind } = pat;
        let kind = match kind {
            PatternKind::Wildcard => ir::PatternKind::Wildcard,
            PatternKind::Ident(ident, sub) => {
                let sub = sub.as_ref().map(|pat| self.lower_pattern(pat));
                ir::PatternKind::Binding(*ident, sub)
            }
            PatternKind::Paren(pat) => return self.lower_pattern_inner(pat),
        };
        ir::Pattern { id: self.lower_node_id(id), span, kind }
    }
}
