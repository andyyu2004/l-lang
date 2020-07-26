use super::ConstId;
use super::{Compiler, Constant, ConstantPool, Executable};
use crate::ast;
use crate::exec::Function;
use crate::ir::DefId;
use crate::lexer::symbol;
use crate::tir;
use crate::typeck::TyCtx;
use indexed_vec::{Idx, IndexVec};
use rustc_hash::FxHashMap;
use std::cell::{Cell, RefCell};

crate struct Compilers<'tcx> {
    compilers: Vec<Compiler<'tcx>>,
    ctx: &'tcx CompilerCtx<'tcx>,
}

impl<'tcx> Compilers<'tcx> {
    pub fn new(ctx: &'tcx CompilerCtx<'tcx>) -> Self {
        Self { compilers: Default::default(), ctx }
    }

    pub fn compile(mut self, tir: &tir::Prog<'tcx>) -> Executable {
        tir.items.values().for_each(|item| self.ctx.assign_const_id(item.id.def_id));
        tir.items.values().for_each(|item| self.compile_item(item));
        match *self.ctx.main_fn.borrow() {
            Some(id) => {
                let constant_pool: ConstantPool =
                    self.ctx.constants.take().into_iter().map(|c| c.unwrap()).collect();
                Executable::with_main_index(constant_pool, id.index())
            }
            None => panic!("no main fn"),
        }
    }

    fn compile_item(&mut self, item: &tir::Item<'tcx>) {
        match &item.kind {
            tir::ItemKind::Fn(_, _, body) => {
                let f = self.compile_fn(body);
                self.ctx.set_const(item.id.def_id, f);
                if item.ident.symbol == symbol::MAIN {
                    let const_id = self.ctx.def_id_to_const_id.borrow()[&item.id.def_id];
                    *self.ctx.main_fn.borrow_mut() = Some(const_id)
                }
            }
        }
    }

    fn with_compiler<R>(&mut self, f: impl FnOnce(&mut Compiler) -> R) -> R {
        self.compilers.push(Compiler::new(self.ctx));
        let compiler = self.compilers.last_mut().unwrap();
        let res = f(compiler);
        self.compilers.pop();
        res
    }

    fn compile_fn(&mut self, body: &tir::Body<'tcx>) -> Function {
        self.with_compiler(|compiler| {
            compiler.compile_body(body);
            compiler.finish()
        })
    }
}

crate struct CompilerCtx<'tcx> {
    pub tcx: TyCtx<'tcx>,
    // mapping of a functions `DefId` to its index in the `ConstantPool`
    pub def_id_to_const_id: RefCell<FxHashMap<DefId, ConstId>>,
    pub main_fn: RefCell<Option<ConstId>>,
    pub constants: RefCell<IndexVec<ConstId, Option<Constant>>>,
    /// counter for assigning `ConstId`s
    constc: Cell<usize>,
}

impl<'tcx> CompilerCtx<'tcx> {
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
    fn assign_const_id(&self, def_id: DefId) {
        let const_id = self.constants.borrow_mut().push(None);
        self.def_id_to_const_id.borrow_mut().insert(def_id, const_id);
    }

    fn set_const(&self, def_id: DefId, value: impl Into<Constant>) {
        let const_id = self.def_id_to_const_id.borrow()[&def_id];
        self.constants.borrow_mut()[const_id] = Some(value.into());
    }

    fn mk_const(&self, value: impl Into<Constant>) -> ConstId {
        self.constants.borrow_mut().push(Some(value.into()))
    }
}
