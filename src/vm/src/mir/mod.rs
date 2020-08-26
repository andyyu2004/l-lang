//! mid-level intermediate representation (control flow graph)

mod build;
mod fmt;
pub use fmt::MirFmt;

use crate::ir;
use crate::span::Span;
use crate::ty::{Const, Ty};
use crate::{ast, mir};
pub use build::build_fn;
use indexed_vec::{Idx, IndexVec};
use std::collections::BTreeMap;
use std::marker::PhantomData;

newtype_index!(BlockId);

pub const RETURN: usize = 0;

#[derive(Clone, Debug)]
pub struct Prog<'tcx> {
    pub bodies: BTreeMap<ir::Id, Body<'tcx>>,
}

impl<'tcx> std::fmt::Display for Prog<'tcx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for body in self.bodies.values() {
            writeln!(f, "{}", body)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Body<'tcx> {
    pub basic_blocks: IndexVec<BlockId, BasicBlock<'tcx>>,
    pub vars: IndexVec<VarId, Var<'tcx>>,
}

impl<'tcx> std::fmt::Display for Body<'tcx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::Formatter::new(f, self).fmt()
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct BasicBlock<'tcx> {
    pub stmts: Vec<mir::Stmt<'tcx>>,
    /// this is optional only for construction
    pub terminator: Option<mir::Terminator<'tcx>>,
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

#[derive(Clone, Debug, PartialEq)]
pub enum StmtKind<'tcx> {
    Assign(Lvalue<'tcx>, Rvalue<'tcx>),
    Nop,
}

newtype_index!(VarId);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Lvalue<'tcx> {
    pub var: VarId,
    pd: PhantomData<&'tcx ()>,
}

impl<'tcx> Lvalue<'tcx> {
    pub fn new(var: VarId) -> Self {
        Self { var, pd: PhantomData }
    }

    /// `VarId` 1 is reserved for return lvalues
    pub fn ret() -> Self {
        Self::new(VarId::new(RETURN))
    }

    /// `VarId` 0 is reserved for the null lvalue (something akin to /dev/null)
    pub fn null() -> Self {
        unimplemented!()
    }
}

impl<'tcx> From<VarId> for Lvalue<'tcx> {
    fn from(var: VarId) -> Self {
        Self::new(var)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Var<'tcx> {
    pub info: SpanInfo,
    pub kind: VarKind,
    pub ty: Ty<'tcx>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VarKind {
    /// mir introduced temporary
    Tmp,
    /// user declared variable
    Local,
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

// this design flattens out recursive expressions into a series of temporaries
#[derive(Clone, Debug, PartialEq)]
pub enum Operand<'tcx> {
    Const(&'tcx Const<'tcx>),
    Ref(Lvalue<'tcx>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Terminator<'tcx> {
    pub info: SpanInfo,
    pub kind: TerminatorKind<'tcx>,
}

/// information of the original source code that was converted into the mir
#[derive(Copy, Clone, Debug, PartialEq)]
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
