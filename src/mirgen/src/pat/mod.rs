mod build;
mod exhaustiveness;

use super::*;
use arena::DroplessArena;
use exhaustiveness::Witness;
use ir::Visitor;
use lcore::queries::Queries;
use lcore::ty::TypeckTables;
use std::ops::Deref;
use thiserror::Error;

crate fn provide(queries: &mut Queries) {
    *queries = Queries { check_patterns, ..*queries }
}

#[derive(Debug, Error)]
enum PatternError<'p, 'tcx> {
    #[error("non-exhaustive match expression\npattern `{0}` not covered")]
    NonexhaustiveMatch(Witness<'p, 'tcx>),
    #[error("redundant pattern")]
    RedundantPattern,
}

/// validate match expressions and patterns in general in the body of `def_id`
fn check_patterns<'tcx>(tcx: TyCtx<'tcx>, def_id: DefId) {
    let body = tcx.defs().body(def_id);
    let tables = tcx.typeck(def_id);
    PatVisitor::new(tcx, tables).visit_body(body)
}

struct PatVisitor<'tcx> {
    tcx: TyCtx<'tcx>,
    tables: &'tcx TypeckTables<'tcx>,
    arena: DroplessArena,
}

struct MatchCtxt<'a, 'tcx> {
    visitor: &'a PatVisitor<'tcx>,
    pat_arena: &'a DroplessArena,
}

impl<'tcx> PatVisitor<'tcx> {
    pub fn new(tcx: TyCtx<'tcx>, tables: &'tcx TypeckTables<'tcx>) -> Self {
        Self { tcx, tables, arena: Default::default() }
    }

    fn with_match_ctxt<R>(&self, f: impl for<'a> FnOnce(MatchCtxt<'a, 'tcx>) -> R) -> R {
        f(MatchCtxt { visitor: self, pat_arena: &self.arena })
    }
}

impl<'a, 'tcx> Deref for MatchCtxt<'a, 'tcx> {
    type Target = PatVisitor<'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.visitor
    }
}

impl<'tcx> Deref for PatVisitor<'tcx> {
    type Target = TyCtx<'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.tcx
    }
}

impl<'tcx> Visitor<'tcx> for PatVisitor<'tcx> {
    fn visit_expr(&mut self, expr: &'tcx ir::Expr<'tcx>) {
        if let ir::ExprKind::Match(scrut, arms, _src) = &expr.kind {
            self.with_match_ctxt(|mcx| mcx.check_match(expr, scrut, arms));
        };
        ir::walk_expr(self, expr);
    }
}
