//! mid-level intermediate representation (control flow graph)

mod build;

use crate::span::Span;
use crate::ty::Const;
use crate::{ast, mir};
pub use build::build_fn;
use indexed_vec::IndexVec;
use std::marker::PhantomData;

newtype_index!(BlockId);

#[derive(Clone, Debug)]
pub struct Body<'tcx> {
    basic_blocks: IndexVec<BlockId, BasicBlock<'tcx>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BasicBlock<'tcx> {
    pub stmts: Vec<mir::Stmt<'tcx>>,
    pub terminator: mir::Terminator<'tcx>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Stmt<'tcx> {
    pub info: SpanData,
    pub kind: mir::StmtKind<'tcx>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum StmtKind<'tcx> {
    Assign(Box<(Lvalue<'tcx>, Rvalue<'tcx>)>),
    Nop,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Lvalue<'tcx> {
    pd: PhantomData<&'tcx ()>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Rvalue<'tcx> {
    /// usage of some variable
    Ref(Lvalue<'tcx>),
    Bin(ast::BinOp, Operand<'tcx>, Operand<'tcx>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operand<'tcx> {
    Const(&'tcx Const<'tcx>),
    Ref(Lvalue<'tcx>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Terminator<'tcx> {
    pub info: SpanData,
    pub kind: TerminatorKind<'tcx>,
}

/// information of the original source code that was converted into the mir
#[derive(Clone, Debug, PartialEq)]
pub struct SpanData {
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
