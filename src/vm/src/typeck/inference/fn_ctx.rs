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
    locals: RefCell<FxHashMap<ir::Id, Ty<'tcx>>>,
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

    pub fn type_pattern(&self, pat: &ir::Pattern, expected: Ty<'tcx>) -> &'tcx tir::Pattern<'tcx> {
        let &ir::Pattern { span, id, ref kind } = pat;
        let (kind, ty) = match kind {
            ir::PatternKind::Wildcard => (tir::PatternKind::Wildcard, expected),
            &ir::PatternKind::Binding(ident, ref sub) => {
                if sub.is_some() {
                    unimplemented!()
                }
                (tir::PatternKind::Binding(ident, None), self.local_ty(pat.id))
            }
        };
        self.write_ty(pat.id, ty);
        self.tcx.alloc_tir(tir::Pattern { span, id, kind, ty })
    }

    pub fn check_block(&self, block: &ir::Block) -> Ty<'tcx> {
        block.stmts.iter().for_each(|stmt| self.check_stmt(stmt));
        match &block.expr {
            Some(expr) => self.check_expr(expr),
            None => self.tcx.types.unit,
        }
    }

    pub fn check_let_stmt(&self, l: &ir::Let) {
        let local_ty = self.local_ty(l.id);
        let init_ty = l.init.as_ref().map(|expr| self.check_expr(expr));
    }

    pub fn local_ty(&self, id: ir::Id) -> Ty<'tcx> {
        self.locals.borrow().get(&id).cloned().expect("no entry for local variable")
    }

    pub fn check_stmt(&self, stmt: &ir::Stmt) {
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
