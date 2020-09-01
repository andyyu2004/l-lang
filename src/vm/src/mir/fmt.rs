//! mir formatter

use crate::ast::BinOp;
use crate::mir::{self, BasicBlock, Body, VarId};
use crate::util;
use std::fmt;
use std::fmt::Write;

/// indentation constant
const INDENT: &str = "    ";

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

    pub fn indent(&mut self) -> fmt::Result {
        write!(self, "{}", INDENT)
    }

    pub fn indentn(&mut self, n: usize) -> fmt::Result {
        for _ in 0..n {
            write!(self, "{}", INDENT)?;
        }
        Ok(())
    }

    pub fn fmt(&mut self) -> fmt::Result {
        writeln!(self, "MIR")?;

        for (id, var) in self.mir.vars.iter_enumerated() {
            writeln!(self, "%{:?}:{} ({:?})", id, var.ty, var.kind)?;
        }

        writeln!(self)?;

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
        let ty = self.mir.vars[lvalue.id].ty;
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
        f.indent()?;
        match self {
            mir::StmtKind::Assign(lvalue, rvalue) => f.fmt_assign(lvalue, rvalue),
            mir::StmtKind::Nop => write!(f, "nop"),
        }?;
        writeln!(f)
    }
}

impl<'tcx> MirFmt<'tcx> for mir::Lvalue<'tcx> {
    fn mir_fmt(&self, f: &mut Formatter<'_, 'tcx>) -> fmt::Result {
        self.id.mir_fmt(f)
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
            mir::Operand::Item(def) => write!(f, "#{:?}", def),
        }
    }
}

impl<'tcx> MirFmt<'tcx> for mir::Terminator<'tcx> {
    fn mir_fmt(&self, f: &mut Formatter<'_, 'tcx>) -> fmt::Result {
        self.kind.mir_fmt(f)
    }
}

impl<'tcx> MirFmt<'tcx> for mir::TerminatorKind<'tcx> {
    fn mir_fmt(&self, fmt: &mut Formatter<'_, 'tcx>) -> fmt::Result {
        fmt.indent()?;
        match self {
            mir::TerminatorKind::Branch(block) => writeln!(fmt, "branch {:?}", block),
            mir::TerminatorKind::Return => writeln!(fmt, "return"),
            mir::TerminatorKind::Unreachable => writeln!(fmt, "unreachable"),
            mir::TerminatorKind::Call { f, args, lvalue, target, unwind } => {
                lvalue.mir_fmt(fmt)?;
                write!(fmt, " <- (")?;
                f.mir_fmt(fmt)?;
                for arg in args {
                    write!(fmt, " ")?;
                    arg.mir_fmt(fmt)?;
                }
                writeln!(fmt, ") -> [{:?}]", target)?;
                writeln!(fmt)
            }
            mir::TerminatorKind::Switch { discr, arms, default } => {
                write!(fmt, "switch ")?;
                discr.mir_fmt(fmt)?;
                writeln!(fmt, " {{")?;
                for (rvalue, block) in arms {
                    fmt.indentn(2)?;
                    write!(fmt, "[")?;
                    rvalue.mir_fmt(fmt)?;
                    writeln!(fmt, " -> {:?}]", block)?;
                }

                fmt.indentn(2)?;
                write!(fmt, "[")?;
                writeln!(fmt, "_ -> {:?}]", default)?;

                writeln!(fmt, "{}}}", INDENT)
            }
        }?;
        writeln!(fmt)
    }
}
