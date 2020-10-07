use super::AstLoweringCtx;
use ast::*;

impl<'ir> AstLoweringCtx<'_, 'ir> {
    /// the id belongs to the `Expr` or the `Ty` or the `Pat`
    pub(super) fn lower_path(&mut self, path: &Path) -> &'ir ir::Path<'ir> {
        let segments = self
            .arena
            .alloc_from_iter(path.segments.iter().map(|seg| self.lower_path_segment(seg)));
        let res = self.lower_res(self.resolver.get_res(path.id));
        let path = ir::Path { span: path.span, segments, res };
        self.alloc(path)
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
