mod error;

use ds::Bitset;
use lcore::mir::*;
use lcore::ty::TyCtx;
use std::ops::Deref;

use self::error::MirError;

pub fn analyze<'a, 'tcx>(tcx: TyCtx<'tcx>, mir: &'a Mir<'tcx>) {
    MirAnalysisCtxt::new(tcx, mir).analyze()
}

struct MirAnalysisCtxt<'a, 'tcx> {
    tcx: TyCtx<'tcx>,
    mir: &'a Mir<'tcx>,
    // bit set to one if initialized
    initialized: Bitset<VarId>,
}

impl<'a, 'tcx> MirAnalysisCtxt<'a, 'tcx> {
    pub fn new(tcx: TyCtx<'tcx>, mir: &'a Mir<'tcx>) -> Self {
        Self { tcx, mir, initialized: Bitset::new(mir.vars.len()) }
    }

    pub fn analyze(&mut self) {
        self.visit_mir(self.mir);
    }
}

impl<'a, 'tcx> MirVisitor<'tcx> for MirAnalysisCtxt<'a, 'tcx> {
    fn visit_stmt(&mut self, stmt: &Stmt<'tcx>) {
        match &stmt.kind {
            StmtKind::Assign(lvalue, _) => {
                self.initialized.set(lvalue.id);
            }
            StmtKind::Retain(_) => {}
            StmtKind::Release(_) => {}
            StmtKind::Nop => {}
        }
        self.walk_stmt(stmt);
    }

    fn visit_lvalue(&mut self, info: SpanInfo, lvalue: &Lvalue<'tcx>) {
        let varkind = self.mir.vars[lvalue.id].kind;
        if varkind == VarKind::Local && !self.initialized.is_set(lvalue.id) {
            self.sess.emit_error(
                info.span,
                MirError::UninitializedVariable(self.mir.vars[lvalue.id].info.span),
            );
        }
    }
}

impl<'tcx> Deref for MirAnalysisCtxt<'_, 'tcx> {
    type Target = TyCtx<'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.tcx
    }
}
