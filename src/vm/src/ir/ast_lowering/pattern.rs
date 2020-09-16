use super::AstLoweringCtx;
use crate::ast::*;
use crate::ir;
use itertools::Itertools;

impl<'ir> AstLoweringCtx<'_, 'ir> {
    pub fn lower_patterns(&mut self, patterns: &[Box<Pattern>]) -> &'ir [ir::Pattern<'ir>] {
        self.arena.alloc_from_iter(patterns.iter().map(|x| self.lower_pattern_inner(x)))
    }

    pub fn lower_pattern(&mut self, pat: &Pattern) -> &'ir ir::Pattern<'ir> {
        self.arena.alloc(self.lower_pattern_inner(pat))
    }

    fn lower_pattern_inner(&mut self, pat: &Pattern) -> ir::Pattern<'ir> {
        let &Pattern { id, span, ref kind } = pat;
        let kind = match kind {
            PatternKind::Wildcard => ir::PatternKind::Wildcard,
            PatternKind::Paren(pat) => return self.lower_pattern_inner(pat),
            PatternKind::Tuple(pats) => ir::PatternKind::Tuple(self.lower_patterns(pats)),
            PatternKind::Variant(_, _) => todo!(),
            PatternKind::Path(_) => todo!(),
            PatternKind::Ident(ident, sub, m) => {
                let sub = sub.as_ref().map(|pat| self.lower_pattern(pat));
                ir::PatternKind::Binding(*ident, sub, *m)
            }
        };
        ir::Pattern { id: self.lower_node_id(id), span, kind }
    }
}
