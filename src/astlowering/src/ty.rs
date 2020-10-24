use super::AstLoweringCtx;
use ast::*;

impl<'ir> AstLoweringCtx<'_, 'ir> {
    pub fn lower_tys(&mut self, tys: &[Box<Ty>]) -> &'ir [ir::Ty<'ir>] {
        self.arena.alloc_from_iter(tys.iter().map(|x| self.lower_ty_inner(x)))
    }

    pub fn lower_ty(&mut self, ty: &Ty) -> &'ir ir::Ty<'ir> {
        self.arena.alloc(self.lower_ty_inner(ty))
    }

    pub(super) fn lower_ty_inner(&mut self, ty: &Ty) -> ir::Ty<'ir> {
        let &Ty { span, id, ref kind } = ty;
        let kind = match kind {
            TyKind::Paren(ty) => return self.lower_ty_inner(ty),
            TyKind::Array(ty) => ir::TyKind::Array(self.lower_ty(ty)),
            TyKind::Tuple(tys) => ir::TyKind::Tuple(self.lower_tys(tys)),
            TyKind::Path(path) => ir::TyKind::Path(self.lower_path(path)),
            TyKind::Fn(params, ret) =>
                ir::TyKind::Fn(self.lower_tys(params), ret.as_ref().map(|ty| self.lower_ty(ty))),
            TyKind::Box(m, ty) => ir::TyKind::Box(*m, self.lower_ty(ty)),
            TyKind::Ptr(ty) => ir::TyKind::Ptr(self.lower_ty(ty)),
            TyKind::Infer => ir::TyKind::Infer,
            TyKind::Err => ir::TyKind::Err,
        };

        ir::Ty { span, id: self.lower_node_id(id), kind }
    }
}
