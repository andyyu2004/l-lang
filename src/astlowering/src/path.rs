use super::AstLoweringCtx;
use ast::*;

impl<'ir> AstLoweringCtx<'_, 'ir> {
    crate fn lower_qpath(&mut self, path: &Path) -> &'ir ir::QPath<'ir> {
        let partial_res = self.resolver.partial_res(path.id);
        if partial_res.unresolved == 0 {
            let path = self.lower_path(path);
            return self.alloc(ir::QPath::Resolved(path));
        }
        todo!()
    }

    crate fn lower_path(&mut self, path: &Path) -> &'ir ir::Path<'ir> {
        let segments = self
            .arena
            .alloc_from_iter(path.segments.iter().map(|seg| self.lower_path_segment(seg)));

        let partial_res = self.resolver.partial_res(path.id);
        assert_eq!(partial_res.unresolved, 0);
        let res = self.lower_res(partial_res.resolved);
        self.alloc(ir::Path { span: path.span, segments, res })
    }

    pub fn lower_path_segment(&mut self, segment: &PathSegment) -> ir::PathSegment<'ir> {
        let &PathSegment { id, ident, ref args } = segment;
        ir::PathSegment {
            ident,
            id: self.lower_node_id(id),
            args: args.as_ref().map(|args| self.lower_generic_args(&args)),
        }
    }

    fn lower_generic_args(&mut self, args: &GenericArgs) -> &'ir ir::GenericArgs<'ir> {
        let args = ir::GenericArgs { span: args.span, args: self.lower_tys(&args.args) };
        self.alloc(args)
    }
}
