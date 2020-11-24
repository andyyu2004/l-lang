use crate::mir::*;

/// trait for mir visitor
/// `walk_*` provides default implementations
/// override `visit_*`
pub trait MirVisitor<'tcx> {
    fn visit_mir(&mut self, mir: &Mir<'tcx>) {
        self.walk_mir(mir)
    }

    fn walk_mir(&mut self, mir: &Mir<'tcx>) {
        for block in &mir.basic_blocks {
            self.visit_basic_block(block);
        }
    }

    fn visit_basic_block(&mut self, block: &BasicBlock<'tcx>) {
        self.walk_basic_block(block)
    }

    fn walk_basic_block(&mut self, block: &BasicBlock<'tcx>) {
        for stmt in &block.stmts {
            self.visit_stmt(stmt);
        }
        self.visit_terminator(block.terminator());
    }

    fn visit_stmt(&mut self, stmt: &Stmt<'tcx>) {
        self.walk_stmt(stmt)
    }

    fn walk_stmt(&mut self, stmt: &Stmt<'tcx>) {
        match &stmt.kind {
            StmtKind::Assign(lvalue, rvalue) => self.visit_assignment(stmt.info, lvalue, rvalue),
            StmtKind::Retain(_) => {}
            StmtKind::Release(_) => {}
            StmtKind::Nop => {}
        }
    }

    fn visit_assignment(&mut self, info: SpanInfo, lvalue: &Lvalue<'tcx>, rvalue: &Rvalue<'tcx>) {
        self.walk_assignment(info, lvalue, rvalue);
    }

    fn walk_assignment(&mut self, info: SpanInfo, lvalue: &Lvalue<'tcx>, rvalue: &Rvalue<'tcx>) {
        self.visit_lvalue(info, lvalue);
        self.visit_rvalue(info, rvalue);
    }

    fn visit_lvalue(&mut self, info: SpanInfo, lvalue: &Lvalue<'tcx>) {
        self.walk_lvalue(info, lvalue);
    }

    fn walk_lvalue(&mut self, _info: SpanInfo, _lvalue: &Lvalue<'tcx>) {
    }

    fn visit_rvalue(&mut self, info: SpanInfo, rvalue: &Rvalue<'tcx>) {
        self.walk_rvalue(info, rvalue);
    }

    fn walk_rvalue(&mut self, info: SpanInfo, rvalue: &Rvalue<'tcx>) {
        match rvalue {
            Rvalue::Box(operand) | Rvalue::Operand(operand) | Rvalue::Unary(_, operand) =>
                self.visit_operand(info, operand),
            Rvalue::Bin(_, l, r) => {
                self.visit_operand(info, l);
                self.visit_operand(info, r);
            }
            Rvalue::Ref(lvalue) | Rvalue::Discriminant(lvalue) => self.visit_lvalue(info, lvalue),
            Rvalue::Closure(_, _) => todo!(),
            Rvalue::Adt { adt, variant_idx, substs, fields } => {
                let (..) = (adt, variant_idx, substs);
                fields.iter().for_each(|field| self.visit_operand(info, field));
            }
        }
    }

    fn visit_operand(&mut self, info: SpanInfo, operand: &Operand<'tcx>) {
        self.walk_operand(info, operand);
    }

    fn walk_operand(&mut self, info: SpanInfo, operand: &Operand<'tcx>) {
        match operand {
            Operand::Lvalue(lvalue) => self.visit_lvalue(info, lvalue),
            Operand::Const(..) => {}
            Operand::Item(..) => {}
        }
    }

    fn visit_terminator(&mut self, terminator: &Terminator<'tcx>) {
        self.walk_terminator(terminator)
    }

    fn walk_terminator(&mut self, terminator: &Terminator<'tcx>) {
        match &terminator.kind {
            TerminatorKind::Branch(_) => {}
            TerminatorKind::Return => {}
            TerminatorKind::Unreachable => {}
            TerminatorKind::Call { f, args, lvalue, target, unwind } => {
                self.visit_lvalue(terminator.info, lvalue);
                self.visit_operand(terminator.info, f);
                args.iter().for_each(|arg| self.visit_operand(terminator.info, arg));
                let (..) = (target, unwind);
            }
            TerminatorKind::Switch { discr, arms, default } => {
                self.visit_operand(terminator.info, discr);
                arms.iter().for_each(|(operand, _)| self.visit_operand(terminator.info, operand));
                let _ = default;
            }
            TerminatorKind::Cond(operand, _, _) => self.visit_operand(terminator.info, operand),
            TerminatorKind::Abort => {}
        };
    }
}
