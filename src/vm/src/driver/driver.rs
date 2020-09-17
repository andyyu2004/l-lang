use super::Session;
use crate::arena::DroplessArena;
use crate::ast::{self, P};
use crate::compiler::{Compiler, Executable, GlobalCompilerCtx};
use crate::core::Arena;
use crate::error::{DiagnosticBuilder, LError, LResult, ParseResult};
use crate::llvm::CodegenCtx;
use crate::pluralize;
use crate::resolve::{Resolver, ResolverArenas, ResolverOutputs};
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
    resolver_arenas: ResolverArenas<'tcx>,
    global_ctx: OnceCell<GlobalCtx<'tcx>>,
    sess: Session,
}

#[macro_export]
macro_rules! pluralize {
    ($x:expr) => {
        if $x != 1 { "s" } else { "" }
    };
}

/// exits if any errors have been reported
macro check_errors($self:expr, $ret:expr) {{
    if $self.sess.has_errors() {
        let errc = $self.sess.err_count();
        e_red_ln!("{} error{} emitted", errc, pluralize!(errc));
        Err(LError::ErrorReported)
    } else {
        Ok($ret)
    }
}}

impl<'tcx> Driver<'tcx> {
    pub fn new(src: &str) -> Self {
        span::GLOBALS
            .with(|globals| *globals.source_map.borrow_mut() = Some(Rc::new(SourceMap::new(src))));
        Self {
            resolver_arenas: Default::default(),
            arena: Default::default(),
            global_ctx: OnceCell::new(),
            sess: Default::default(),
        }
    }

    pub fn lex(&self) -> LResult<Vec<Tok>> {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex();
        Ok(tokens)
    }

    /// used for testing parsing
    pub fn parse_expr(&self) -> Option<P<ast::Expr>> {
        let tokens = self.lex().unwrap();
        let mut parser = Parser::new(&self.sess, tokens);
        parser.parse_expr().map_err(|err| err.emit()).ok()
    }

    pub fn parse(&self) -> LResult<P<ast::Prog>> {
        let tokens = self.lex()?;
        let mut parser = Parser::new(&self.sess, tokens);
        let ast = parser.parse();
        check_errors!(self, ast.unwrap())
    }

    pub fn gen_ir(&'tcx self) -> LResult<(&'tcx ir::Prog<'tcx>, ResolverOutputs)> {
        let ast = self.parse()?;
        let mut resolver = Resolver::new(&self.sess, &self.resolver_arenas);
        resolver.resolve(&ast);
        let lctx = AstLoweringCtx::new(&self.arena, &mut resolver);
        let ir = lctx.lower_prog(&ast);
        info!("{:#?}", ir);
        let resolutions = resolver.complete();
        Ok((ir, resolutions))
    }

    fn with_tcx_and_ir<R>(
        &'tcx self,
        f: impl FnOnce(TyCtx<'tcx>, &'tcx ir::Prog<'tcx>) -> R,
    ) -> LResult<R> {
        let (ir, mut resolutions) = self.gen_ir()?;
        let defs = self.arena.alloc(std::mem::take(&mut resolutions.defs));
        let gcx = self.global_ctx.get_or_init(|| GlobalCtx::new(&self.arena, &defs, &self.sess));
        let ret = gcx.enter_tcx(|tcx| f(tcx, &ir));
        check_errors!(self, ret)
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
        check_errors!(self, (cctx, main_fn.unwrap()))
    }

    pub fn llvm_exec(&'tcx self) -> LResult<i64> {
        let (ctx, main_fn) = self.llvm_compile()?;
        // execution
        let jit = ctx.module.create_jit_execution_engine(OptimizationLevel::None).unwrap();
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
