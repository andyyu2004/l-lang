use super::AstLoweringCtx;
use lc_ast::*;

impl<'ir> AstLoweringCtx<'_, 'ir> {
    pub(crate) fn lower_qpath(&mut self, path: &Path) -> &'ir ir::QPath<'ir> {
        let partial_res = self.resolver.partial_res(path.id);
        let unresolved_start = path.segments.len() - partial_res.unresolved;

        let base_path = self.partial_lower_path(path);
        let mut qpath = self.alloc(ir::QPath::Resolved(base_path));

        // create type relative qpath
        for unresolved_segment in &path.segments[unresolved_start..] {
            // this loop should not run at all if there are no unresolved segments
            let segment = self.arena.alloc(self.lower_path_segment(unresolved_segment));
            let ty = self.mk_ty_path(base_path.span, qpath);
            qpath = self.alloc(ir::QPath::TypeRelative(ty, segment));
        }

        debug!("qpath {}", qpath);
        qpath
    }

    pub(crate) fn lower_path(&mut self, path: &Path) -> &'ir ir::Path<'ir> {
        // this following line checks that this path is a fully resolved path
        // otherwise, `lower_qpath` should be used
        self.resolver.full_res(path.id);
        self.partial_lower_path(path)
    }

    /// lowers the resolved portion of the path
    pub(crate) fn partial_lower_path(&mut self, path: &Path) -> &'ir ir::Path<'ir> {
        let partial_res = self.resolver.partial_res(path.id);
        let unresolved_start = path.segments.len() - partial_res.unresolved;
        let res = self.lower_res(partial_res.resolved);

        let segments = self.arena.alloc_from_iter(
            path.segments[..unresolved_start].iter().map(|seg| self.lower_path_segment(seg)),
        );
        self.alloc(ir::Path { span: path.span, segments, res })
    }

    pub fn lower_path_segment(&mut self, segment: &PathSegment) -> ir::PathSegment<'ir> {
        let &PathSegment { id: _, ident, ref args } = segment;
        ir::PathSegment { ident, args: args.as_ref().map(|args| self.lower_generic_args(args)) }
    }

    fn lower_generic_args(&mut self, args: &GenericArgs) -> &'ir ir::GenericArgs<'ir> {
        let args = ir::GenericArgs { span: args.span, args: self.lower_tys(&args.args) };
        self.alloc(args)
    }
}
