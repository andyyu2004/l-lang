use super::FnCtx;
use ir::{self, Visitor};
use lcore::ty::{TyCtx, UpvarId};
use rustc_hash::FxHashSet;

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    /// write the upvars mentioned by `closure` to `TypeckOutputs`
    crate fn record_upvars(&mut self, closure: &ir::Expr, body: &ir::Body) {
        let locals = LocalsVisitor::find_locals(body);
        let upvar_visitor = UpvarVisitor::new(self.tcx, closure.id, &locals);
        let upvars = upvar_visitor.resolve_upvars(body);
        self.tables.borrow_mut().record_upvar_capture_for_closure(closure.id, upvars);
    }
}

/// finds all the variables the closure references that are nonlocal
struct UpvarVisitor<'a, 'tcx> {
    #[allow(unused)]
    tcx: TyCtx<'tcx>,
    closure_id: ir::Id,
    locals: &'a FxHashSet<ir::Id>,
    upvars: FxHashSet<UpvarId>,
}

impl<'a, 'tcx> UpvarVisitor<'a, 'tcx> {
    fn new(tcx: TyCtx<'tcx>, closure_id: ir::Id, locals: &'a FxHashSet<ir::Id>) -> Self {
        Self { locals, tcx, closure_id, upvars: Default::default() }
    }

    fn resolve_upvars(mut self, body: &ir::Body) -> FxHashSet<UpvarId> {
        self.visit_body(body);
        self.upvars
    }
}

impl<'a, 'tcx> Visitor<'_> for UpvarVisitor<'a, 'tcx> {
    /// the only way to syntactically reference a variable is a path expression
    fn visit_path(&mut self, path: &ir::Path) {
        if let ir::Res::Local(var_id) = path.res {
            if !self.locals.contains(&var_id) {
                self.upvars.insert(UpvarId { closure_id: self.closure_id, var_id });
            }
        }
    }
}

/// collects the ids of all variables local to a body
/// this includes all local declarations and parameters
#[derive(Default)]
struct LocalsVisitor {
    locals: FxHashSet<ir::Id>,
}

impl LocalsVisitor {
    fn find_locals(body: &ir::Body) -> FxHashSet<ir::Id> {
        let mut v = Self::default();
        v.visit_body(body);
        v.locals
    }
}

impl ir::Visitor<'_> for LocalsVisitor {
    /// ultimately, the only way to introduce a name in L is via PatternKind::Binding
    fn visit_pat(&mut self, pat: &ir::Pattern) {
        if let ir::PatternKind::Binding(..) = pat.kind {
            self.locals.insert(pat.id);
        }
        ir::walk_pat(self, pat);
    }
}
