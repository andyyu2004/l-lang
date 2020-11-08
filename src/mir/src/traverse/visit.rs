use lcore::mir::*;

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
        match &stmt.kind {
            StmtKind::Assign(lvalue, rvalue) => self.visit_assignment(lvalue, rvalue),
            StmtKind::Retain(_) => {}
            StmtKind::Release(_) => {}
            StmtKind::Nop => {}
        }
    }

    fn visit_assignment(&mut self, lvalue: &Lvalue<'tcx>, rvalue: &Rvalue<'tcx>) {
        self.walk_assignment(lvalue, rvalue);
    }

    fn walk_assignment(&mut self, lvalue: &Lvalue<'tcx>, rvalue: &Rvalue<'tcx>) {
        self.visit_lvalue(lvalue);
        self.visit_rvalue(rvalue);
    }

    fn visit_lvalue(&mut self, lvalue: &Lvalue<'tcx>) {
        self.walk_lvalue(lvalue);
    }

    fn walk_lvalue(&mut self, _lvalue: &Lvalue<'tcx>) {
    }

    fn visit_rvalue(&mut self, rvalue: &Rvalue<'tcx>) {
        self.walk_rvalue(rvalue);
    }

    fn walk_rvalue(&mut self, rvalue: &Rvalue<'tcx>) {
        match rvalue {
            Rvalue::Box(operand) | Rvalue::Operand(operand) | Rvalue::Unary(_, operand) =>
                self.visit_operand(operand),
            Rvalue::Bin(_, l, r) => {
                self.visit_operand(l);
                self.visit_operand(r);
            }
            Rvalue::Ref(lvalue) | Rvalue::Discriminant(lvalue) => self.visit_lvalue(lvalue),
            Rvalue::Closure(_, _) => todo!(),
            Rvalue::Adt { adt, variant_idx, substs, fields } => {
                let (..) = (adt, variant_idx, substs);
                fields.iter().for_each(|field| self.visit_operand(field));
            }
        }
    }

    fn visit_operand(&mut self, operand: &Operand<'tcx>) {
        self.walk_operand(operand);
    }

    fn walk_operand(&mut self, operand: &Operand<'tcx>) {
        match operand {
            Operand::Lvalue(lvalue) => self.visit_lvalue(lvalue),
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
                self.visit_lvalue(lvalue);
                self.visit_operand(f);
                args.iter().for_each(|arg| self.visit_operand(arg));
                let (..) = (target, unwind);
            }
            TerminatorKind::Switch { discr, arms, default } => {
                self.visit_operand(discr);
                arms.iter().for_each(|(operand, _)| self.visit_operand(operand));
                let _ = default;
            }
            TerminatorKind::Cond(operand, _, _) => self.visit_operand(operand),
            TerminatorKind::Abort => {}
        };
    }
}
