mod build;
mod exhaustiveness;

use super::*;
use arena::DroplessArena;
use ir::Visitor;
use lcore::queries::Queries;
use lcore::ty::TypeckTables;

crate fn provide(queries: &mut Queries) {
    *queries = Queries { check_patterns, ..*queries }
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
    tcx: TyCtx<'tcx>,
    arena: &'a DroplessArena,
}

impl<'tcx> PatVisitor<'tcx> {
    pub fn new(tcx: TyCtx<'tcx>, tables: &'tcx TypeckTables<'tcx>) -> Self {
        Self { tcx, tables, arena: Default::default() }
    }

    fn with_match_ctxt<R>(&self, f: impl for<'a> FnOnce(MatchCtxt<'a, 'tcx>) -> R) -> R {
        f(MatchCtxt { tcx: self.tcx, arena: &self.arena })
    }
}

impl<'tcx> Visitor<'tcx> for PatVisitor<'tcx> {
    fn visit_expr(&mut self, expr: &'tcx ir::Expr<'tcx>) {
        if let ir::ExprKind::Match(scrut, arms, _src) = &expr.kind {
            self.with_match_ctxt(|mcx| mcx.check_match(scrut, arms));
        };
        ir::walk_expr(self, expr);
    }
}
