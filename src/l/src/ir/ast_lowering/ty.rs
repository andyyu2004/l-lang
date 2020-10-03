use super::AstLoweringCtx;
use crate::ast::*;
use crate::ir;
use itertools::Itertools;

impl<'ir> AstLoweringCtx<'_, 'ir> {
    pub fn lower_tys(&mut self, tys: &[Box<Ty>]) -> &'ir [ir::Ty<'ir>] {
        self.arena.ir.alloc_from_iter(tys.iter().map(|x| self.lower_ty_inner(x)))
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
            TyKind::Infer => ir::TyKind::Infer,
            TyKind::Ptr(m, ty) => ir::TyKind::Ptr(*m, self.lower_ty(ty)),
        };

        ir::Ty { span, id: self.lower_node_id(id), kind }
    }
}
