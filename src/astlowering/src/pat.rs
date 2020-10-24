use super::AstLoweringCtx;
use ast::*;

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
            PatternKind::Box(pat) => ir::PatternKind::Box(self.lower_pattern(pat)),
            PatternKind::Paren(pat) => return self.lower_pattern_inner(pat),
            PatternKind::Tuple(pats) => ir::PatternKind::Tuple(self.lower_patterns(pats)),
            PatternKind::Variant(path, patterns) =>
                ir::PatternKind::Variant(self.lower_qpath(path), self.lower_patterns(patterns)),
            PatternKind::Path(path) => ir::PatternKind::Path(self.lower_qpath(path)),
            PatternKind::Struct(path, fields) =>
                ir::PatternKind::Struct(self.lower_qpath(path), self.lower_field_pats(fields)),
            PatternKind::Ident(ident, sub, m) => {
                let sub = sub.as_ref().map(|pat| self.lower_pattern(pat));
                ir::PatternKind::Binding(*ident, sub, *m)
            }
            PatternKind::Lit(expr) => ir::PatternKind::Lit(self.lower_expr(expr)),
            PatternKind::Wildcard => ir::PatternKind::Wildcard,
        };
        ir::Pattern { id: self.lower_node_id(id), span, kind }
    }

    fn lower_field_pats(&mut self, field_pats: &[FieldPat]) -> &'ir [ir::FieldPat<'ir>] {
        self.arena.alloc_from_iter(field_pats.iter().map(|field| self.lower_field_pat(field)))
    }

    fn lower_field_pat(&mut self, field_pat: &FieldPat) -> ir::FieldPat<'ir> {
        let &FieldPat { span, ident, ref pat } = field_pat;
        ir::FieldPat { span, ident, pat: self.lower_pattern(pat) }
    }
}
