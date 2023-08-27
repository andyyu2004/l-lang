//! mid-level intermediate representation (control flow graph)
mod fmt;
mod mirty;
mod traverse;
mod visit;

pub use mirty::{LvalueTy, MirTy};
pub use traverse::{postorder, preorder, rpo};
pub use visit::MirVisitor;

use crate::mir;
use crate::ty::{AdtTy, Const, List, Projection, SubstsRef, Ty, TyCtx};
use ir::{DefId, VariantIdx};
use lc_ast::Mutability;
use lc_index::{newtype_index, Idx, IndexVec};
use lc_span::Span;
use std::ops::{Deref, DerefMut};

newtype_index! {
    pub struct BlockId {
        DEBUG_FORMAT = "bb{}",
        const ENTRY_BLOCK = 0,
    }
}

newtype_index! {
    pub struct VarId {
        DEBUG_FORMAT = "{}",
        const RET_VAR = 0,
    }
}

/// top level mir structure
/// approximately analogous to a tir::Body
#[derive(Clone, Debug, PartialEq, Default)]
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
    Nop,
}

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
        Some(self.cmp(other))
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

/// we introduce a new enumeration of unary ops at mir level
/// as there are less valid operations now
/// namely, dereferences and references become explicit expressions
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

impl From<lc_ast::UnaryOp> for UnaryOp {
    fn from(op: lc_ast::UnaryOp) -> Self {
        match op {
            lc_ast::UnaryOp::Neg => Self::Neg,
            lc_ast::UnaryOp::Not => Self::Not,
            lc_ast::UnaryOp::Deref | lc_ast::UnaryOp::Ref =>
                panic!("invalid unary op at mir level `{}`", op),
        }
    }
}

impl std::fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOp::Neg => write!(f, "-"),
            UnaryOp::Not => write!(f, "!"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Rvalue<'tcx> {
    /// box x
    Box(Operand<'tcx>),
    /// x
    Operand(Operand<'tcx>),
    /// - x
    Unary(UnaryOp, Operand<'tcx>),
    /// + x y
    Bin(lc_ast::BinOp, Operand<'tcx>, Operand<'tcx>),
    /// &x
    Ref(Lvalue<'tcx>),
    /// reads the discriminant of an enum
    Discriminant(Lvalue<'tcx>),
    /// TODO temporary representation, incomplete
    Closure(Ty<'tcx>),
    Adt {
        adt: &'tcx AdtTy,
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
    Item(DefId, SubstsRef<'tcx>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Terminator<'tcx> {
    pub info: SpanInfo,
    pub kind: TerminatorKind<'tcx>,
}

impl<'tcx> Terminator<'tcx> {
    pub fn successors(&self) -> Vec<BlockId> {
        match self.kind {
            TerminatorKind::Cond(_, a, b) => vec![a, b],
            TerminatorKind::Branch(block) => vec![block],
            TerminatorKind::Call { target, unwind, .. } =>
                Some(target).into_iter().chain(unwind).collect(),
            TerminatorKind::Switch { ref arms, default, .. } =>
                arms.iter().map(|(_, b)| *b).chain(Some(default)).collect(),
            TerminatorKind::Abort | TerminatorKind::Return | TerminatorKind::Unreachable => vec![],
        }
    }

    pub fn successors_mut(&mut self) -> Vec<&mut BlockId> {
        match &mut self.kind {
            TerminatorKind::Cond(_, a, b) => vec![a, b],
            TerminatorKind::Branch(target) | TerminatorKind::Call { target, unwind: None, .. } =>
                vec![target],
            TerminatorKind::Call { target, unwind: Some(unwind), .. } => vec![target, unwind],
            TerminatorKind::Switch { arms, default, .. } =>
                arms.iter_mut().map(|(_, b)| b).chain(Some(default)).collect(),
            TerminatorKind::Abort | TerminatorKind::Return | TerminatorKind::Unreachable => vec![],
        }
    }
}

/// information of the original source code that was converted into the mir
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SpanInfo {
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TerminatorKind<'tcx> {
    /// unconditional branch
    Branch(BlockId),
    /// conditional branch
    Cond(Operand<'tcx>, BlockId, BlockId),
    Return,
    Unreachable,
    Abort,
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

// instead of these could maybe just implement deref from mir -> basic blocks
impl<'tcx> Deref for Mir<'tcx> {
    type Target = IndexVec<BlockId, BasicBlock<'tcx>>;

    fn deref(&self) -> &Self::Target {
        &self.basic_blocks
    }
}

impl<'tcx> DerefMut for Mir<'tcx> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.basic_blocks
    }
}

impl<'a, 'tcx> IntoIterator for &'a Mir<'tcx> {
    type IntoIter = <&'a IndexVec<BlockId, BasicBlock<'tcx>> as IntoIterator>::IntoIter;
    type Item = &'a BasicBlock<'tcx>;

    fn into_iter(self) -> Self::IntoIter {
        self.basic_blocks.iter()
    }
}
