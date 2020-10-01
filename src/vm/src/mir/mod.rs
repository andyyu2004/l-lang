//! mid-level intermediate representation (control flow graph)

mod build;
mod fmt;
mod interpret;
mod opt;
mod refcount;
pub mod traversal;
mod typecheck;
mod visit;

pub use fmt::MirFmt;

use crate::ast::{self, Ident, Mutability, Visibility};
use crate::dataflow;
use crate::ir::{self, DefId, FieldIdx, VariantIdx};
use crate::mir;
use crate::span::Span;
use crate::tir::{self, TirCtx};
use crate::ty::{AdtTy, Const, List, Projection, SubstsRef, Ty};
pub use build::{build_enum_ctors, build_fn};
use indexed_vec::{Idx, IndexVec};
use rustc_hash::FxHashMap;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use visit::Visitor;

newtype_index!(BlockId);

pub const RETURN: usize = 0;

/// mir analyses go here
/// dataflow etc...
pub fn analyse<'a, 'tcx>(mir: &mir::Mir<'tcx>, ctx: &TirCtx<'a, 'tcx>) {
    dataflow::check_assignments(mir, ctx);
}

pub fn validate<'a, 'tcx>(mir: &mir::Mir<'tcx>, ctx: &TirCtx<'a, 'tcx>) {
    typecheck::check(mir, ctx);
}

/// top level mir structure
/// approximately analogous to a tir::Body
#[derive(Clone, Debug, PartialEq)]
pub struct Mir<'tcx> {
    pub basic_blocks: IndexVec<BlockId, BasicBlock<'tcx>>,
    pub vars: IndexVec<VarId, Var<'tcx>>,
    pub argc: usize,
}

impl<'tcx> Mir<'tcx> {
    /// returns the `VarId` of all the parameters/arguments of the `Body`
    pub fn arg_iter(&self) -> impl Iterator<Item = VarId> + ExactSizeIterator {
        // 0 is reserved for returns
        // so 1..1 + argc are the parameters
        (1..1 + self.argc).map(VarId::new)
    }

    /// iterates over all non arg and non return vars
    pub fn var_iter(&self) -> impl Iterator<Item = VarId> + ExactSizeIterator {
        // 0 is reserved for returns
        // so 1..1 + argc are the parameters
        (1 + self.argc..self.vars.len()).map(VarId::new)
    }
}

impl<'tcx> std::fmt::Display for Mir<'tcx> {
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
    Retain(VarId),
    Release(VarId),
    Nop,
}

const RET_VAR: VarId = VarId(0);

newtype_index!(VarId);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Lvalue<'tcx> {
    pub id: VarId,
    pub projs: &'tcx List<Projection<'tcx>>,
}

impl<'tcx> Ord for Lvalue<'tcx> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl<'tcx> PartialOrd for Lvalue<'tcx> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl<'tcx> Lvalue<'tcx> {
    pub fn new(var: VarId) -> Self {
        Self { id: var, projs: List::empty() }
    }

    /// `VarId` 0 is reserved for return lvalues
    pub fn ret() -> Self {
        Self::new(RET_VAR)
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
    pub mtbl: Mutability,
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
    Upvar,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Rvalue<'tcx> {
    /// x
    Operand(Operand<'tcx>),
    /// - x
    Unary(ast::UnaryOp, Operand<'tcx>),
    /// + x y
    Bin(ast::BinOp, Operand<'tcx>, Operand<'tcx>),
    /// returns (uninit) memory of `Ty`
    Box(Ty<'tcx>),
    /// &x
    Ref(Lvalue<'tcx>),
    Closure(Ty<'tcx>, mir::Mir<'tcx>),
    Adt {
        adt: &'tcx AdtTy<'tcx>,
        variant_idx: VariantIdx,
        substs: SubstsRef<'tcx>,
        fields: Vec<Operand<'tcx>>,
    },
}

// this design flattens out recursive expressions into a series of temporaries
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Operand<'tcx> {
    Lvalue(Lvalue<'tcx>),
    Const(&'tcx Const<'tcx>),
    Item(DefId),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Terminator<'tcx> {
    pub info: SpanInfo,
    pub kind: TerminatorKind<'tcx>,
}

impl<'tcx> Terminator<'tcx> {
    pub fn successors(&self) -> Vec<BlockId> {
        match self.kind {
            TerminatorKind::Branch(block)
            | TerminatorKind::Call { target: block, unwind: None, .. } => vec![block],
            TerminatorKind::Return | TerminatorKind::Unreachable => vec![],
            TerminatorKind::Call { target, unwind: Some(unwind), .. } => vec![target, unwind],
            TerminatorKind::Cond(_, a, b) => vec![a, b],
            TerminatorKind::Switch { ref arms, default, .. } =>
                arms.iter().map(|(_, b)| *b).collect(),
        }
    }
}

/// information of the original source code that was converted into the mir
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SpanInfo {
    span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TerminatorKind<'tcx> {
    /// unconditional branch
    Branch(BlockId),
    /// conditional branch
    Cond(Operand<'tcx>, BlockId, BlockId),
    Return,
    Unreachable,
    Call {
        f: Operand<'tcx>,
        args: Vec<Operand<'tcx>>,
        /// lvalue to write the function return value to
        lvalue: Lvalue<'tcx>,
        /// the block to branch to after the call (if no unwind)
        target: BlockId,
        unwind: Option<BlockId>,
    },
    /// if `discr` evaluates to the `Operand`, then the respective block is executed
    Switch {
        discr: Operand<'tcx>,
        arms: Vec<(Operand<'tcx>, BlockId)>,
        default: BlockId,
    },
}
