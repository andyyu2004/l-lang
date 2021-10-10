use crate::*;
use lc_ast::*;
use lc_span::kw;

impl<'ir> AstLoweringCtx<'_, 'ir> {
    pub fn lower_tys(&mut self, tys: &[Box<Ty>]) -> &'ir [ir::Ty<'ir>] {
        self.arena.alloc_from_iter(tys.iter().map(|x| self.lower_ty_inner(x)))
    }

    pub fn lower_ty(&mut self, ty: &Ty) -> &'ir ir::Ty<'ir> {
        self.arena.alloc(self.lower_ty_inner(ty))
    }

    pub(crate) fn lower_ty_inner(&mut self, ty: &Ty) -> ir::Ty<'ir> {
        let &Ty { span, id, ref kind } = ty;
        let kind = match kind {
            TyKind::Fn(params, ret) =>
                ir::TyKind::Fn(self.lower_tys(params), ret.as_ref().map(|ty| self.lower_ty(ty))),
            TyKind::Box(ty) => ir::TyKind::Box(self.lower_ty(ty)),
            TyKind::Paren(ty) => return self.lower_ty_inner(ty),
            TyKind::Array(ty) => ir::TyKind::Array(self.lower_ty(ty)),
            TyKind::Tuple(tys) => ir::TyKind::Tuple(self.lower_tys(tys)),
            TyKind::Path(path) => ir::TyKind::Path(self.lower_qpath(path)),
            TyKind::Ptr(ty) => ir::TyKind::Ptr(self.lower_ty(ty)),
            TyKind::ImplicitSelf => {
                let res = self.resolver.full_res(ty.id);
                let res = self.lower_res(res);
                // we resolve the implicit type of self to the type of the surrounding impl
                ir::TyKind::Path(self.alloc(ir::QPath::Resolved(self.alloc(ir::Path {
                    res,
                    span: ty.span,
                    segments: arena_vec![self; ir::PathSegment {ident: Ident::new(ty.span, kw::USelf), args: None}],
                }))))
            }
            TyKind::Infer => ir::TyKind::Infer,
            TyKind::Err => ir::TyKind::Err,
        };

        ir::Ty { span, id: self.lower_node_id(id), kind }
    }
}
