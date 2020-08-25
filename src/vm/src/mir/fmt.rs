//! mir formatter

use crate::{
    ast::BinOp, mir::{self, BasicBlock}
};
use std::fmt;
use std::fmt::Write;

pub struct Formatter<'a, 'tcx> {
    writer: &'a mut dyn Write,
    mir: &'a mir::Body<'tcx>,
}

impl<'a, 'tcx> Write for Formatter<'a, 'tcx> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.writer.write_str(s)
    }
}

trait MirFmt<'tcx> {
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
        write!(f, "%{:?}", self.var)?;
        let ty = f.mir.vars[self.var].ty;
        write!(f, ":{} ← ", ty)
    }
}

impl<'tcx> MirFmt<'tcx> for mir::Var<'tcx> {
    fn mir_fmt(&self, f: &mut Formatter<'_, 'tcx>) -> fmt::Result {
        match self.kind {
            mir::VarKind::Tmp => write!(f, "tmpvar"),
            mir::VarKind::Var => write!(f, "{}", self.info.span.to_string()),
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
