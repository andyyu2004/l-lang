use super::AstLoweringCtx;
use crate::ast::*;
use crate::ir;
use std::marker::PhantomData;

impl<'ir> AstLoweringCtx<'_, 'ir> {
    pub(super) fn lower_path(&mut self, path: &Path) -> &'ir ir::Path<'ir> {
        // just handle the local variable case for now
        assert!(path.segments.len() == 1);
        let seg = &path.segments[0];
        let segments = vec![ir::PathSegment {
            ident: seg.ident,
            id: self.lower_node_id(seg.id),
            pd: PhantomData,
        }];
        let segments = self.arena.alloc_from_iter(segments);
        let res = self.lower_res(self.resolver.get_res(seg.id));
        let path = ir::Path { span: path.span, segments, res };
        self.arena.alloc(path)
    }
}
