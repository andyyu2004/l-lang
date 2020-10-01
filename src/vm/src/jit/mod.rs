mod native;

use crate::gc::*;
use crate::lexer::symbol;
use crate::llvm;
use crate::mir;
use inkwell::context::Context;
use inkwell::execution_engine::ExecutionEngine;
use inkwell::types::{BasicTypeEnum, PointerType};
use inkwell::values::*;
use inkwell::OptimizationLevel;
use itertools::Itertools;
use llvm::CodegenCtx;
use std::ops::Deref;

pub struct Runtime<G> {
    gc: G,
}

pub struct JitCtx<'a, 'tcx, G> {
    runtime: Runtime<G>,
    engine: ExecutionEngine<'tcx>,
    cctx: &'a llvm::CodegenCtx<'tcx>,
    stack: Vec<BasicValueEnum<'tcx>>,
}

impl<'a, 'tcx, G> JitCtx<'a, 'tcx, G>
where
    G: GarbageCollector<'tcx>,
{
    pub fn new(cctx: &'a llvm::CodegenCtx<'tcx>, gc: G) -> Self {
        Self {
            runtime: Runtime { gc },
            cctx,
            engine: cctx.module.create_jit_execution_engine(OptimizationLevel::None).unwrap(),
            stack: vec![],
        }
    }

    pub fn jit_compile_fn(&self, body: &mir::Mir<'tcx>) -> FunctionValue<'tcx> {
        todo!()
    }

    pub fn jit_body(&self, llfn: FunctionValue<'tcx>, body: &'tcx mir::Mir<'tcx>) {
        todo!()
    }

    fn jit_operand(&mut self, operand: &mir::Operand<'tcx>) -> Value<'tcx> {
        todo!()
    }

    fn jit_terminator(&mut self, terminator: &mir::Terminator<'tcx>) {
        match &terminator.kind {
            mir::TerminatorKind::Branch(_) => {}
            mir::TerminatorKind::Return => {}
            mir::TerminatorKind::Unreachable => {}
            mir::TerminatorKind::Call { f, args, lvalue, target, unwind } => {
                let f = self.jit_operand(f).as_fn();
                let args = vec![];
                // let args = args.iter().map(|arg| self.jit_operand(arg)).collect_vec();
                let ret = self.build_call(f, &args, "fcall").try_as_basic_value().left().unwrap();
                self.stack.push(ret);
            }
            mir::TerminatorKind::Switch { discr, arms, default } => {}
        }
    }
}

enum Value<'tcx> {
    Int(i64),
    Function(FunctionValue<'tcx>),
}

impl<'tcx> Value<'tcx> {
    fn as_fn(self) -> FunctionValue<'tcx> {
        match self {
            Self::Function(f) => f,
            _ => panic!(),
        }
    }
}

impl<'a, 'tcx, G> Deref for JitCtx<'a, 'tcx, G> {
    type Target = llvm::CodegenCtx<'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.cctx
    }
}
