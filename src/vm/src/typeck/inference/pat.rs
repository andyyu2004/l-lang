use super::FnCtx;
use crate::error::TypeResult;
use crate::ty::*;
use crate::typeck::{TyCtx, TypeckTables};
use crate::{ast, ir, tir};

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    /// typechecks a given pattern with its expected type
    pub fn check_pat(&mut self, pat: &ir::Pattern, ty: Ty<'tcx>) -> Ty<'tcx> {
        // note that the type is recorded for each identifier as well as the whole pattern
        let pat_ty = match &pat.kind {
            ir::PatternKind::Wildcard => ty,
            ir::PatternKind::Binding(ident, _) => self.write_ty(ident.id, ty),
        };
        self.write_ty(pat.id, ty)
    }
}
