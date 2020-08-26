use super::CodegenCtx;
use crate::mir;
use crate::mir::*;
use inkwell::values::*;
use rustc_hash::FxHashMap;
use std::ops::Deref;

pub(super) struct FnCtx<'a, 'tcx> {
    cctx: &'a CodegenCtx<'tcx>,
    body: &'tcx mir::Body<'tcx>,
    function: FunctionValue<'tcx>,
    vars: FxHashMap<VarId, Var<'tcx>>,
}

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    pub fn new(
        ctx: &'a CodegenCtx<'tcx>,
        body: &'tcx mir::Body<'tcx>,
        function: FunctionValue<'tcx>,
    ) -> Self {
        Self { cctx: ctx, body, function, vars: Default::default() }
    }

    crate fn codegen_body(&mut self, body: &'tcx mir::Body<'tcx>) {
        for (id, &var) in body.vars.iter_enumerated() {
            self.alloca(id, var);
        }
        for basic_block in &body.basic_blocks {
            self.compile_basic_block(basic_block);
        }
    }

    fn alloca(&mut self, id: VarId, var: Var<'tcx>) -> PointerValue<'tcx> {
        self.vars.insert(id, var);
        self.builder.build_alloca(self.llvm_ty(var.ty), &var.to_string())
    }

    fn compile_basic_block(&mut self, basic_block: &mir::BasicBlock) {
        for stmt in &basic_block.stmts {
            self.compile_stmt(stmt);
        }
    }

    fn compile_stmt(&mut self, stmt: &mir::Stmt) {
        match &stmt.kind {
            mir::StmtKind::Assign(box (lvalue, rvalue)) => {}
            mir::StmtKind::Nop => {}
        }
    }
}

impl<'a, 'tcx> Deref for FnCtx<'a, 'tcx> {
    type Target = CodegenCtx<'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.cctx
    }
}
