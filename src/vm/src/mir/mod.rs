//! mid-level intermediate representation (control flow graph)

mod build;

use crate::span::Span;
use crate::ty::{Const, Ty};
use crate::{ast, mir};
pub use build::build_fn;
use indexed_vec::{Idx, IndexVec};
use std::fmt::{self, Display, Formatter};
use std::marker::PhantomData;

newtype_index!(BlockId);

#[derive(Clone, Debug)]
pub struct Body<'tcx> {
    basic_blocks: IndexVec<BlockId, BasicBlock<'tcx>>,
}

impl<'tcx> Display for Body<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (i, block) in self.basic_blocks.iter_enumerated() {
            writeln!(f, "basic_block {:?}:", i)?;
            write!(f, "{}", block)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct BasicBlock<'tcx> {
    pub stmts: Vec<mir::Stmt<'tcx>>,
    /// this is optional only for construction
    pub terminator: Option<mir::Terminator<'tcx>>,
}

impl<'tcx> Display for BasicBlock<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for stmt in &self.stmts {
            writeln!(f, "\t{}", stmt)?;
        }
        writeln!(f, "\t{}", self.terminator())
    }
}

impl<'tcx> BasicBlock<'tcx> {
    pub fn terminator(&self) -> &mir::Terminator<'tcx> {
        self.terminator.as_ref().unwrap()
    }

    pub fn terminator_mut(&mut self) -> &mut mir::Terminator<'tcx> {
        self.terminator.as_mut().unwrap()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Stmt<'tcx> {
    pub info: SpanInfo,
    pub kind: mir::StmtKind<'tcx>,
}

impl<'tcx> Display for Stmt<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum StmtKind<'tcx> {
    Assign(Box<(Lvalue<'tcx>, Rvalue<'tcx>)>),
    Nop,
}

impl<'tcx> Display for StmtKind<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            StmtKind::Assign(box (lvalue, rvalue)) => write!(f, "{} ← {}", lvalue, rvalue),
            StmtKind::Nop => write!(f, "nop"),
        }
    }
}

newtype_index!(VarId);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Lvalue<'tcx> {
    var: VarId,
    pd: PhantomData<&'tcx ()>,
}

impl<'tcx> Lvalue<'tcx> {
    pub fn new(var: VarId) -> Self {
        Self { var, pd: PhantomData }
    }

    /// `VarId` 0 is reserved for return lvalues
    pub fn ret() -> Self {
        Self::new(VarId::new(0))
    }
}

impl<'tcx> From<VarId> for Lvalue<'tcx> {
    fn from(var: VarId) -> Self {
        Self::new(var)
    }
}

impl<'tcx> Display for Lvalue<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "%{:?}", self.var)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Var<'tcx> {
    info: SpanInfo,
    kind: VarKind,
    ty: Ty<'tcx>,
}

impl Display for Var<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.kind {
            VarKind::Tmp => write!(f, "tmpvar"),
            VarKind::Var => write!(f, "{}", self.info.span.to_string()),
            VarKind::Arg => todo!(),
            VarKind::Ret => write!(f, "retvar"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum VarKind {
    /// mir introduced temporary
    Tmp,
    /// user declared variable
    Var,
    /// function argument
    Arg,
    /// location of return value.
    Ret,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Rvalue<'tcx> {
    /// usage of some variable
    Use(Operand<'tcx>),
    Bin(ast::BinOp, Operand<'tcx>, Operand<'tcx>),
}

impl<'tcx> Display for Rvalue<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Rvalue::Use(operand) => write!(f, "{}", operand),
            Rvalue::Bin(op, lhs, rhs) => write!(f, "{} {} {}", lhs, op, rhs),
        }
    }
}

// this design flattens out recursive expressions into a series of temporaries
#[derive(Clone, Debug, PartialEq)]
pub enum Operand<'tcx> {
    Const(&'tcx Const<'tcx>),
    Ref(Lvalue<'tcx>),
}

impl<'tcx> Display for Operand<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Const(c) => write!(f, "{}", c),
            Operand::Ref(lvalue) => write!(f, "{}", lvalue),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Terminator<'tcx> {
    pub info: SpanInfo,
    pub kind: TerminatorKind<'tcx>,
}

/// information of the original source code that was converted into the mir
#[derive(Clone, Debug, PartialEq)]
pub struct SpanInfo {
    span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TerminatorKind<'tcx> {
    Branch(BlockId),
    Return,
    Unreachable,
    Call {
        f: Operand<'tcx>,
        args: Vec<Operand<'tcx>>,
    },
    Switch {
        discr: Operand<'tcx>,
        // i32 is placeholder type for now
        arms: Vec<(i32, BlockId)>,
        default: Option<BlockId>,
    },
}

impl<'tcx> Display for Terminator<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl<'tcx> Display for TerminatorKind<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TerminatorKind::Branch(_) => todo!(),
            TerminatorKind::Return => write!(f, "return"),
            TerminatorKind::Unreachable => write!(f, "unreachable"),
            TerminatorKind::Call { f, args } => todo!(),
            TerminatorKind::Switch { discr, arms, default } => todo!(),
        }
    }
}
