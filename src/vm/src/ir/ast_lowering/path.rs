use super::AstLoweringCtx;
use crate::ast::*;
use crate::ir::{self, Res};
use crate::span::Span;
use std::marker::PhantomData;

impl<'ir> AstLoweringCtx<'_, 'ir> {
    /// the id belongs to the `Expr` or the `Ty` or the `Pat`
    pub(super) fn lower_path(&mut self, id: NodeId, path: &Path) -> &'ir ir::Path<'ir> {
        let segments = self
            .arena
            .ir
            .alloc_from_iter(path.segments.iter().map(|seg| self.lower_path_segment(seg)));
        let res = self.lower_res(self.resolver.get_res(id));
        let path = ir::Path { span: path.span, segments, res };
        self.alloc(path)
    }

    pub fn lower_path_segment(&mut self, segment: &PathSegment) -> ir::PathSegment<'ir> {
        let &PathSegment { id, ident, args } = segment;
        ir::PathSegment { ident, id: self.lower_node_id(id), pd: PhantomData }
    }
}
