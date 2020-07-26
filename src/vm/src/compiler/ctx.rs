use super::ConstId;
use super::{Compiler, Constant, ConstantPool, Executable};
use crate::ast;
use crate::exec::{CodeBuilder, Function};
use crate::ir::{DefId, LocalId};
use crate::lexer::symbol;
use crate::tir;
use crate::typeck::TyCtx;
use indexed_vec::{Idx, IndexVec};
use rustc_hash::FxHashMap;
use std::cell::{Cell, RefCell};
use std::ops::{Deref, DerefMut};

// ctx specific to a frame
crate struct FrameCtx<'tcx> {
    pub(super) code: CodeBuilder,
    pub(super) locals: Vec<LocalId>,
    pub(super) gctx: &'tcx GlobalCompilerCtx<'tcx>,
}

impl<'tcx> Deref for FrameCtx<'tcx> {
    type Target = CodeBuilder;

    fn deref(&self) -> &Self::Target {
        &self.code
    }
}

impl<'tcx> DerefMut for FrameCtx<'tcx> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.code
    }
}

impl<'tcx> FrameCtx<'tcx> {
    pub fn new(gctx: &'tcx GlobalCompilerCtx<'tcx>) -> Self {
        Self { gctx, code: Default::default(), locals: Default::default() }
    }
}

crate struct GlobalCompilerCtx<'tcx> {
    pub tcx: TyCtx<'tcx>,
    // mapping of a functions `DefId` to its index in the `ConstantPool`
    pub def_id_to_const_id: RefCell<FxHashMap<DefId, ConstId>>,
    pub main_fn: RefCell<Option<ConstId>>,
    pub constants: RefCell<IndexVec<ConstId, Option<Constant>>>,
    /// counter for assigning `ConstId`s
    constc: Cell<usize>,
}

impl<'tcx> GlobalCompilerCtx<'tcx> {
    pub fn new(tcx: TyCtx<'tcx>) -> Self {
        Self {
            tcx,
            constants: Default::default(),
            def_id_to_const_id: Default::default(),
            main_fn: Default::default(),
            constc: Cell::new(0),
        }
    }

    /// assigns a `ConstId` for a constant that is not yet known
    pub(super) fn assign_const_id(&self, def_id: DefId) {
        let const_id = self.constants.borrow_mut().push(None);
        self.def_id_to_const_id.borrow_mut().insert(def_id, const_id);
    }

    pub(super) fn set_const(&self, def_id: DefId, value: impl Into<Constant>) {
        let const_id = self.def_id_to_const_id.borrow()[&def_id];
        self.constants.borrow_mut()[const_id] = Some(value.into());
    }

    pub(super) fn mk_const(&self, value: impl Into<Constant>) -> ConstId {
        self.constants.borrow_mut().push(Some(value.into()))
    }
}
