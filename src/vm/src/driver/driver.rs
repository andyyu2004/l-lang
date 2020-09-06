use super::Session;
use crate::arena::DroplessArena;
use crate::ast::{self, P};
use crate::compiler::{Compiler, Executable, GlobalCompilerCtx};
use crate::core::Arena;
use crate::error::{DiagnosticBuilder, LError, LResult};
use crate::llvm::CodegenCtx;
use crate::resolve::{Resolver, ResolverOutputs};
use crate::typeck::{GlobalCtx, TyCtx};
use crate::{exec, ir, lexer, mir, parser, span, tir};
use exec::VM;
use inkwell::{context::Context as LLVMCtx, values::FunctionValue, OptimizationLevel};
use ir::AstLoweringCtx;
use lexer::{symbol, Lexer, Tok};
use once_cell::unsync::OnceCell;
use parser::Parser;
use span::SourceMap;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Driver<'tcx> {
    arena: Arena<'tcx>,
    ir_arena: DroplessArena,
    global_ctx: OnceCell<GlobalCtx<'tcx>>,
    session: Session,
}

impl<'tcx> Driver<'tcx> {
    pub fn new(src: &str) -> Self {
        span::GLOBALS
            .with(|globals| *globals.source_map.borrow_mut() = Some(Rc::new(SourceMap::new(src))));
        Self {
            arena: Default::default(),
            ir_arena: Default::default(),
            global_ctx: OnceCell::new(),
            session: Default::default(),
        }
    }

    pub fn lex(&self) -> LResult<Vec<Tok>> {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex();
        Ok(tokens)
    }

    /// used for testing parsing
    pub fn parse_expr(&self) -> LResult<P<ast::Expr>> {
        let tokens = self.lex()?;
        let mut parser = Parser::new(tokens);
        let expr = parser.parse_expr()?;
        Ok(expr)
    }

    pub fn parse(&self) -> LResult<P<ast::Prog>> {
        let tokens = self.lex()?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        Ok(ast)
    }

    pub fn gen_ir<'ir>(&'ir self) -> LResult<(ir::Prog<'ir>, ResolverOutputs)> {
        let ast = self.parse()?;
        let mut resolver = Resolver::resolve(&ast);
        let lctx = AstLoweringCtx::new(&self.ir_arena, &mut resolver);
        let ir = lctx.lower_prog(&ast);
        info!("{:#?}", ir);
        let resolutions = resolver.complete();
        Ok((ir, resolutions))
    }

    fn with_tcx_and_ir<R>(
        &'tcx self,
        f: impl for<'ir> FnOnce(TyCtx<'tcx>, &'ir ir::Prog<'tcx>) -> R,
    ) -> LResult<R> {
        let (ir, mut resolutions) = self.gen_ir()?;
        let defs = self.arena.alloc(std::mem::take(&mut resolutions.defs));
        let gcx = self.global_ctx.get_or_init(|| GlobalCtx::new(&self.arena, &defs, &self.session));
        let ret = gcx.enter_tcx(|tcx| f(tcx, &ir));
        if self.session.has_errors() {
            let errc = self.session.err_count();
            if errc == 1 {
                e_red_ln!("{} error emitted", errc)
            } else {
                e_red_ln!("{} errors emitted", errc)
            }
            Err(LError::ErrorReported)
        } else {
            Ok(ret)
        }
    }

    pub fn gen_tir(&'tcx self) -> LResult<tir::Prog<'tcx>> {
        self.with_tcx_and_ir(|tcx, ir| tcx.build_tir(ir))
    }

    pub fn gen_mir(&'tcx self) -> LResult<mir::Prog<'tcx>> {
        self.with_tcx_and_ir(|tcx, ir| tcx.build_mir(ir))
    }

    pub fn llvm_compile(&'tcx self) -> LResult<(CodegenCtx, FunctionValue<'tcx>)> {
        let mir = self.arena.alloc(self.gen_mir()?);
        let gcx = self.global_ctx.get().unwrap();
        let llvm_ctx = LLVMCtx::create();
        let mut cctx = gcx.enter_tcx(|tcx| CodegenCtx::new(tcx, self.arena.alloc(llvm_ctx)));
        let main_fn = cctx.codegen(&mir);
        Ok((cctx, main_fn))
    }

    pub fn llvm_exec(&'tcx self) -> LResult<i64> {
        let (ctx, main_fn) = self.llvm_compile()?;
        // execution
        let jit = ctx.module.create_jit_execution_engine(OptimizationLevel::Aggressive).unwrap();
        let val = unsafe { jit.run_function(main_fn, &[]) };
        Ok(val.as_int(true) as i64)
    }

    // pub fn compile(&'tcx self) -> LResult<Executable> {
    //     let tir = self.gen_mir()?;
    //     println!("{}", tir);
    //     let gcx = self.global_ctx.get().unwrap();

    //     let cctx = gcx.enter_tcx(|tcx| self.arena.alloc(GlobalCompilerCtx::new(tcx)));
    //     let executable = Compiler::new(cctx).compile(&tir);
    //     println!("{}", executable);
    //     Ok(executable)
    // }

    // pub fn exec(&'tcx self) -> LResult<exec::Val> {
    //     let executable = self.compile()?;
    //     let mut vm = VM::with_default_gc(executable);
    //     let value = vm.run()?;
    //     Ok(value)
    // }
}
