use crate::mir::{self, BlockId};
use crate::ty::ConstKind;
use mir::build::ENTRY_BLOCK;

struct Interpreter<'mir, 'tcx> {
    stack: Vec<Frame<'mir, 'tcx>>,
    sp: usize,
}

impl<'mir, 'tcx> Interpreter<'mir, 'tcx> {
    pub fn new(body: &'mir mir::Mir<'tcx>) -> Self {
        Self { stack: vec![Frame::new(body)], sp: 0 }
    }

    fn step(&mut self) {
        let frame = &self.stack[self.sp];
        let (bp, ip) = frame.ip;
        let bb = &frame.body.basic_blocks[bp];
        match bb.stmts.get(ip) {
            Some(stmt) => self.interpret_stmt(stmt),
            None => self.interpret_terminator(bb.terminator()),
        }
    }

    fn interpret_stmt(&mut self, stmt: &mir::Stmt<'tcx>) {
        match &stmt.kind {
            mir::StmtKind::Assign(lvalue, rvalue) => {
                let rvalue = self.interpret_rvalue(rvalue);
                let lvalue = self.interpret_lvalue(lvalue);
            }
            mir::StmtKind::Nop => {}
            mir::StmtKind::Retain(_) => {}
            mir::StmtKind::Release(_) => {}
        }
    }

    fn interpret_lvalue(&mut self, lvalue: &mir::Lvalue<'tcx>) {
    }

    fn interpret_rvalue(&mut self, rvalue: &mir::Rvalue<'tcx>) {
        match rvalue {
            mir::Rvalue::Operand(operand) => self.interpret_operand(operand),
            mir::Rvalue::Unary(_, _) => {}
            mir::Rvalue::Bin(_, _, _) => {}
            mir::Rvalue::Box(_) => {}
            mir::Rvalue::Ref(_) => {}
            mir::Rvalue::Closure(_, _) => {}
            mir::Rvalue::Adt { adt, variant_idx, substs, fields } => {}
            mir::Rvalue::Discriminant(_) => {}
        }
    }

    fn interpret_operand(&mut self, operand: &mir::Operand<'tcx>) {
        match operand {
            mir::Operand::Lvalue(_) => {}
            mir::Operand::Const(c) => match c.kind {
                ConstKind::Float(_) => {}
                ConstKind::Int(_) => {}
                ConstKind::Bool(_) => {}
                ConstKind::Unit => {}
            },
            mir::Operand::Item(_) => {}
        }
    }

    fn interpret_terminator(&mut self, terminator: &mir::Terminator<'tcx>) {
        match &terminator.kind {
            mir::TerminatorKind::Branch(_) => {}
            mir::TerminatorKind::Return => {}
            mir::TerminatorKind::Unreachable => {}
            mir::TerminatorKind::Call { f, args, lvalue, target, unwind } => {}
            mir::TerminatorKind::Switch { discr, arms, default } => {}
            mir::TerminatorKind::Cond(_, _, _) => {}
            mir::TerminatorKind::Abort => {}
        }
    }

    fn interpret_body(body: &mir::Mir<'tcx>) {
    }
}

enum Value<'tcx> {
    Int(i64),
    Bool(bool),
    Function(mir::Mir<'tcx>),
}

struct Frame<'mir, 'tcx> {
    body: &'mir mir::Mir<'tcx>,
    /// (block, stmt_idx)
    ip: (BlockId, usize),
}

impl<'mir, 'tcx> Frame<'mir, 'tcx> {
    pub fn new(body: &'mir mir::Mir<'tcx>) -> Self {
        Self { body, ip: (ENTRY_BLOCK, 0) }
    }
}
