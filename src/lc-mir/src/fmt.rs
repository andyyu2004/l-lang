//! mir formatter

use crate::TyCtx;
use lc_ast::BinOp;
use lc_core::mir;
use lc_core::ty::Projection;
use std::fmt;
use std::fmt::Write;

/// indentation constant
const INDENT: &str = "    ";

pub struct Formatter<'a, 'tcx> {
    tcx: TyCtx<'tcx>,
    writer: &'a mut dyn Write,
    mir: &'a mir::Mir<'tcx>,
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
    pub fn new(tcx: TyCtx<'tcx>, writer: &'a mut dyn Write, mir: &'a mir::Mir<'tcx>) -> Self {
        Self { tcx, writer, mir }
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
            id.mir_fmt(self)?;
            writeln!(self, ":{} ({:?})", var.ty, var.kind,)?;
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
            mir::StmtKind::Retain(var) => {
                write!(f, "retain ")?;
                var.mir_fmt(f)
            }
            mir::StmtKind::Release(var) => {
                write!(f, "release ")?;
                var.mir_fmt(f)
            }
            mir::StmtKind::Nop => write!(f, "nop"),
        }?;
        writeln!(f)
    }
}

impl<'tcx> MirFmt<'tcx> for mir::Lvalue<'tcx> {
    fn mir_fmt(&self, f: &mut Formatter<'_, 'tcx>) -> fmt::Result {
        self.id.mir_fmt(f)?;
        for p in self.projs {
            match p {
                Projection::Field(field, _) => write!(f, ".{:?}", field)?,
                Projection::Deref => write!(f, ".*")?,
                Projection::PointerCast(ty) => write!(f, " as {}", ty)?,
            }
        }
        if !self.projs.is_empty() {
            write!(f, " (")?;
            self.id.mir_fmt(f)?;
            for p in self.projs {
                match p {
                    Projection::Field(_, ty) => write!(f, "->{}", ty)?,
                    Projection::Deref => {}
                    Projection::PointerCast(_) => {}
                };
            }
            write!(f, ")")?;
        }
        Ok(())
    }
}

impl<'tcx> MirFmt<'tcx> for mir::VarId {
    fn mir_fmt(&self, f: &mut Formatter<'_, 'tcx>) -> fmt::Result {
        let var = f.mir.vars[*self];
        match var.kind {
            mir::VarKind::Tmp | mir::VarKind::Ret | mir::VarKind::Upvar =>
                write!(f, "{}{:?}", var, self),
            _ => write!(f, "{}", var),
        }
    }
}

impl<'tcx> MirFmt<'tcx> for mir::Rvalue<'tcx> {
    fn mir_fmt(&self, f: &mut Formatter<'_, 'tcx>) -> fmt::Result {
        match self {
            mir::Rvalue::Operand(operand) => operand.mir_fmt(f),
            mir::Rvalue::Bin(op, lhs, rhs) => f.fmt_bin(*op, lhs, rhs),
            mir::Rvalue::Adt { adt, .. } => write!(f, "{} {{ .. }}", adt.ident),
            mir::Rvalue::Ref(lvalue) => {
                write!(f, "&")?;
                lvalue.mir_fmt(f)
            }
            mir::Rvalue::Box(operand) => {
                write!(f, "box ")?;
                operand.mir_fmt(f)
            }
            mir::Rvalue::Unary(op, operand) => {
                write!(f, "{}", op)?;
                operand.mir_fmt(f)
            }
            mir::Rvalue::Closure(_, _body) => write!(f, "<closure>"),
            mir::Rvalue::Discriminant(lvalue) => {
                write!(f, "discr ")?;
                lvalue.mir_fmt(f)
            }
        }
    }
}

impl<'tcx> MirFmt<'tcx> for mir::Operand<'tcx> {
    fn mir_fmt(&self, f: &mut Formatter<'_, 'tcx>) -> fmt::Result {
        match self {
            mir::Operand::Const(c) => write!(f, "{}", c),
            mir::Operand::Lvalue(lvalue) => lvalue.mir_fmt(f),
            mir::Operand::Item(def, _ty) => write!(f, "#{}", def),
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
                let ty = fmt.mir.vars[lvalue.id].ty;
                write!(fmt, ":{} ← call ", ty)?;
                f.mir_fmt(fmt)?;
                write!(fmt, "(")?;
                args.get(0).map(|arg| arg.mir_fmt(fmt));
                if args.len() > 1 {
                    for arg in &args[1..] {
                        write!(fmt, ", ")?;
                        arg.mir_fmt(fmt)?;
                    }
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
            mir::TerminatorKind::Cond(cond, a, b) => {
                write!(fmt, "if ")?;
                cond.mir_fmt(fmt)?;
                write!(fmt, " then {:?}", a)?;
                writeln!(fmt, " else {:?}", b)
            }
            mir::TerminatorKind::Abort => writeln!(fmt, "abort"),
        }?;
        writeln!(fmt)
    }
}
