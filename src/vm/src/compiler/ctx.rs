use super::{Compiler, ConstantPool, Executable};
use crate::ast;
use crate::exec::Function;
use crate::tir;
use crate::typeck::TyCtx;

crate struct CompilerCtx<'tcx> {
    tcx: TyCtx<'tcx>,
    compilers: Vec<Compiler>,
}

impl<'tcx> CompilerCtx<'tcx> {
    pub fn new(tcx: TyCtx<'tcx>) -> Self {
        Self { tcx, compilers: Default::default() }
    }

    pub fn compile(&mut self, tir: &tir::Prog<'tcx>) -> Executable {
        // just stupidly compile and run the first item only for now
        let item = tir.items.values().next().unwrap();
        let function = self.compile_item(item);
        println!("{}", function.code);
        Executable::new(ConstantPool::default(), function)
        // for item in tir.items.values() {
        //     let function = self.compile_item(item);
        // }
    }

    fn compile_item(&mut self, item: &tir::Item<'tcx>) -> Function {
        match &item.kind {
            tir::ItemKind::Fn(_, _, body) => self.compile_fn(body),
        }
    }

    fn compile_fn(&mut self, body: &tir::Body<'tcx>) -> Function {
        self.compilers.push(Compiler::new());
        let compiler = self.compilers.last_mut().unwrap();
        compiler.compile_expr(body.expr);
        let function = compiler.finish();
        self.compilers.pop();
        function
    }
}
