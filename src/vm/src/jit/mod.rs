mod fcx;

use crate::error::LResult;
use crate::gc::*;
use crate::ir::{self, DefId};
use crate::lexer::symbol;
use crate::llvm;
use crate::mir;
use crate::typeck::TyCtx;
use fcx::Fcx;
use inkwell::execution_engine::ExecutionEngine;
use inkwell::values::*;
use inkwell::OptimizationLevel;
use itertools::Itertools;
use rustc_hash::FxHashMap;
use std::ops::Deref;

pub struct JitCtx<'a, 'tcx, G> {
    gc: G,
    tcx: TyCtx<'tcx>,
    engine: ExecutionEngine<'tcx>,
    cctx: &'a llvm::CodegenCtx<'tcx>,
    stack: Vec<BasicValueEnum<'tcx>>,
    mir: FxHashMap<DefId, &'tcx mir::Body<'tcx>>,
}

impl<'a, 'tcx, G> JitCtx<'a, 'tcx, G>
where
    G: GarbageCollector<'tcx>,
{
    pub fn new(cctx: &'a llvm::CodegenCtx<'tcx>, gc: G) -> LResult<Self> {
        let mut jit = Self {
            gc,
            cctx,
            tcx: cctx.tcx,
            engine: cctx.module.create_jit_execution_engine(OptimizationLevel::None).unwrap(),
            stack: Default::default(),
            mir: Default::default(),
        };
        jit.init()?;
        Ok(jit)
    }

    fn init(&mut self) -> LResult<()> {
        self.collect_mir()?;
        Ok(())
    }

    fn collect_mir(&mut self) -> LResult<()> {
        for (&def, item) in &self.tcx.ir.items {
            match &item.kind {
                ir::ItemKind::Fn(..) =>
                    if let Ok(mir) = self.tcx.build_mir(def) {
                        self.mir.insert(def, mir);
                    },
                _ => {}
            }
        }
        self.tcx.sess.check_for_errors()
    }

    pub fn run_jit(&mut self) -> i32 {
        dbg!(&self.mir);
        let tcx = self.tcx;
        let ir = tcx.ir;
        let main_id = ir.entry_id.unwrap();
        let mir = self.mir[&main_id];
        let llfn = todo!();
        let fcx = Fcx::new(self, llfn, mir);
        todo!()
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
