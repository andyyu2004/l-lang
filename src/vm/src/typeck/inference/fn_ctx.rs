use super::InferCtx;
use crate::error::TypeResult;
use crate::ir::{self, DefId};
use crate::span::Span;
use crate::tir;
use crate::ty::{SubstRef, Ty, TyConv, TyKind};
use crate::typeck::{TyCtx, TypeckTables};
use ir::FnSig;
use itertools::Itertools;
use rustc_hash::FxHashMap;
use std::{cell::RefCell, ops::Deref};

crate struct FnCtx<'a, 'tcx> {
    infcx: &'a InferCtx<'a, 'tcx>,
    locals: FxHashMap<ir::Id, Ty<'tcx>>,
}

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    pub fn new(infcx: &'a InferCtx<'a, 'tcx>) -> Self {
        Self { infcx, locals: Default::default() }
    }
}

impl<'a, 'tcx> Deref for FnCtx<'a, 'tcx> {
    type Target = InferCtx<'a, 'tcx>;
    fn deref(&self) -> &Self::Target {
        &self.infcx
    }
}

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    pub fn expect_eq(&self, span: Span, expected: Ty<'tcx>, actual: Ty<'tcx>) {
        /// handle and report the error here
        if let Err(err) = self.at(span).equate(expected, actual) {
            println!("{}", err)
        }
    }

    pub fn check_let_stmt(&mut self, l: &ir::Let) {
        let ty = l.ty.map(|ty| self.lower_ty(ty)).unwrap_or(self.new_infer_var());
        let pat_ty = self.check_pat(l.pat, ty);
        self.expect_eq(l.span, ty, pat_ty);
        let init_ty = l.init.as_ref().map(|expr| self.check_expr(expr));
        if let Some(init_ty) = init_ty {
            self.expect_eq(l.span, init_ty, ty)
        }
    }

    pub fn def_local(&mut self, id: ir::Id, ty: Ty<'tcx>) {
        self.locals.insert(id, ty);
    }

    pub fn local_ty(&self, id: ir::Id) -> Ty<'tcx> {
        self.locals.get(&id).cloned().expect("no entry for local variable")
    }

    pub fn check_stmt(&mut self, stmt: &ir::Stmt) {
        match &stmt.kind {
            ir::StmtKind::Let(l) => self.check_let_stmt(l),
            ir::StmtKind::Expr(expr) => {
                self.check_expr(expr);
            }
            ir::StmtKind::Semi(expr) => {
                self.check_expr(expr);
            }
        }
    }
}

impl<'a, 'tcx> TyConv<'tcx> for FnCtx<'a, 'tcx> {
    fn tcx(&self) -> TyCtx<'tcx> {
        self.tcx
    }
}
