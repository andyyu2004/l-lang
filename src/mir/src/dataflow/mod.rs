mod error;

use ast::Mutability;
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

impl<'a, 'tcx> MirAnalysisCtxt<'a, 'tcx> {
    fn is_init(&self, lvalue: &Lvalue<'tcx>) -> bool {
        !self.is_uninit(lvalue)
    }

    fn is_uninit(&self, lvalue: &Lvalue<'tcx>) -> bool {
        // we only need to check `lvalue.id` as all the projections of an lvalue will
        // also be uninit if the variable itself is uninit
        let varkind = self.mir.vars[lvalue.id].kind;
        varkind == VarKind::Local && !self.initialized.is_set(lvalue.id)
    }
}

impl<'a, 'tcx> MirVisitor<'tcx> for MirAnalysisCtxt<'a, 'tcx> {
    fn visit_stmt(&mut self, stmt: &Stmt<'tcx>) {
        match &stmt.kind {
            StmtKind::Assign(lvalue, _) => {
                // only have to check `lvalue.id` as its projections inherits its mutability
                let var = self.mir.vars[lvalue.id];
                // if the variable is uninitialized, then we consider it an
                // initialization not an assignment
                if self.is_init(lvalue) && var.mtbl == Mutability::Imm {
                    self.sess.emit_error(
                        stmt.info.span,
                        MirError::AssignmentToImmutableVar(var.info.span),
                    );
                }
                self.initialized.set(lvalue.id);
            }
            StmtKind::Retain(..) => {}
            StmtKind::Release(..) => {}
            StmtKind::Nop => {}
        }
        self.walk_stmt(stmt);
    }

    fn visit_lvalue(&mut self, info: SpanInfo, lvalue: &Lvalue<'tcx>) {
        if self.is_uninit(lvalue) {
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
