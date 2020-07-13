use super::AstLoweringCtx;
use crate::ast::*;
use crate::ir;
use itertools::Itertools;

impl<'ir> AstLoweringCtx<'_, 'ir> {
    crate fn lower_ty(&mut self, ty: &Ty) -> &'ir ir::Ty<'ir> {
        self.arena.alloc(self.lower_ty_inner(ty))
    }

    pub(super) fn lower_ty_inner(&mut self, ty: &Ty) -> ir::Ty<'ir> {
        todo!()
    }
}
