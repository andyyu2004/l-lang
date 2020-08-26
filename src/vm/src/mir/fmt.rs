//! mir formatter

use crate::ast::BinOp;
use crate::mir::{self, BasicBlock, Body, VarId};
use std::fmt;
use std::fmt::Write;

impl<'tcx> Body<'tcx> {
    pub fn var_name(&self, var: VarId) -> String {
        let mut s = String::new();
        var.mir_fmt(&mut Formatter::new(&mut s, self)).unwrap();
        s
    }
}

pub struct Formatter<'a, 'tcx> {
    writer: &'a mut dyn Write,
    mir: &'a mir::Body<'tcx>,
}

impl<'a, 'tcx> Write for Formatter<'a, 'tcx> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.writer.write_str(s)
    }
}

pub trait MirFmt<'tcx> {
    fn mir_fmt(&self, f: &mut Formatter<'_, 'tcx>) -> fmt::Result;
}

impl<'a, 'tcx> Formatter<'a, 'tcx> {
    pub fn new(writer: &'a mut dyn Write, body: &'a mir::Body<'tcx>) -> Self {
        Self { writer, mir: body }
    }

    pub fn fmt(&mut self) -> fmt::Result {
        for (i, block) in self.mir.basic_blocks.iter_enumerated() {
            writeln!(self.writer, "basic_block {:?}:", i)?;
            block.mir_fmt(self)?;
        }
        writeln!(self)
    }

    fn fmt_assign(
        &mut self,
        lvalue: &mir::Lvalue<'tcx>,
        rvalue: &mir::Rvalue<'tcx>,
    ) -> fmt::Result {
        lvalue.mir_fmt(self)?;
        let ty = self.mir.vars[lvalue.var].ty;
        write!(self, ":{} ← ", ty)?;
        rvalue.mir_fmt(self)
    }

    fn fmt_bin(
        &mut self,
        op: BinOp,
        lhs: &mir::Operand<'tcx>,
        rhs: &mir::Operand<'tcx>,
    ) -> fmt::Result {
        lhs.mir_fmt(self)?;
        write!(self, " {} ", op)?;
        rhs.mir_fmt(self)
    }
}

impl<'tcx> MirFmt<'tcx> for mir::BasicBlock<'tcx> {
    fn mir_fmt(&self, f: &mut Formatter<'_, 'tcx>) -> fmt::Result {
        for stmt in &self.stmts {
            stmt.mir_fmt(f)?;
        }
        self.terminator().mir_fmt(f)
    }
}

impl<'tcx> MirFmt<'tcx> for mir::Stmt<'tcx> {
    fn mir_fmt(&self, f: &mut Formatter<'_, 'tcx>) -> fmt::Result {
        self.kind.mir_fmt(f)
    }
}

impl<'tcx> MirFmt<'tcx> for mir::StmtKind<'tcx> {
    fn mir_fmt(&self, f: &mut Formatter<'_, 'tcx>) -> fmt::Result {
        write!(f, "    ")?;
        match self {
            mir::StmtKind::Assign(box (lvalue, rvalue)) => f.fmt_assign(lvalue, rvalue),
            mir::StmtKind::Nop => write!(f, "nop"),
        }?;
        writeln!(f)
    }
}

impl<'tcx> MirFmt<'tcx> for mir::Lvalue<'tcx> {
    fn mir_fmt(&self, f: &mut Formatter<'_, 'tcx>) -> fmt::Result {
        self.var.mir_fmt(f)
    }
}

impl<'tcx> MirFmt<'tcx> for mir::VarId {
    fn mir_fmt(&self, f: &mut Formatter<'_, 'tcx>) -> fmt::Result {
        let var = f.mir.vars[*self];
        match var.kind {
            mir::VarKind::Tmp => write!(f, "tmp{:?}", self),
            mir::VarKind::Local => write!(f, "{}", var.info.span.to_string()),
            mir::VarKind::Arg => todo!(),
            mir::VarKind::Ret => write!(f, "retvar"),
        }
    }
}

// this implementation is used for giving names for llvm
impl<'tcx> std::fmt::Display for mir::Var<'tcx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            mir::VarKind::Tmp => write!(f, "tmp"),
            mir::VarKind::Local => write!(f, "{}", self.info.span.to_string()),
            mir::VarKind::Arg => todo!(),
            mir::VarKind::Ret => write!(f, "retvar"),
        }
    }
}

impl<'tcx> MirFmt<'tcx> for mir::Rvalue<'tcx> {
    fn mir_fmt(&self, f: &mut Formatter<'_, 'tcx>) -> fmt::Result {
        match self {
            mir::Rvalue::Use(operand) => operand.mir_fmt(f),
            mir::Rvalue::Bin(op, lhs, rhs) => f.fmt_bin(*op, lhs, rhs),
        }
    }
}

impl<'tcx> MirFmt<'tcx> for mir::Operand<'tcx> {
    fn mir_fmt(&self, f: &mut Formatter<'_, 'tcx>) -> fmt::Result {
        match self {
            mir::Operand::Const(c) => write!(f, "{}", c),
            mir::Operand::Ref(lvalue) => lvalue.mir_fmt(f),
        }
    }
}

impl<'tcx> MirFmt<'tcx> for mir::Terminator<'tcx> {
    fn mir_fmt(&self, f: &mut Formatter<'_, 'tcx>) -> fmt::Result {
        self.kind.mir_fmt(f)
    }
}

impl<'tcx> MirFmt<'tcx> for mir::TerminatorKind<'tcx> {
    fn mir_fmt(&self, f: &mut Formatter<'_, 'tcx>) -> fmt::Result {
        match self {
            mir::TerminatorKind::Branch(_) => todo!(),
            mir::TerminatorKind::Return => write!(f, "return"),
            mir::TerminatorKind::Unreachable => write!(f, "unreachable"),
            mir::TerminatorKind::Call { f, args } => todo!(),
            mir::TerminatorKind::Switch { discr, arms, default } => todo!(),
        }
    }
}
