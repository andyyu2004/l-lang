use crate::arena::DroplessArena;
use crate::ast::{self, P};
use crate::compiler::{CompilerCtx, Compilers, Executable};
use crate::core::Arena;
use crate::error::{DiagnosticBuilder, LError, LResult};
use crate::resolve::{Resolver, ResolverOutputs};
use crate::typeck::GlobalCtx;
use crate::{exec, ir, lexer, parser, span, tir};
use exec::VM;
use ir::AstLoweringCtx;
use lexer::{Lexer, Tok};
use once_cell::unsync::OnceCell;
use parser::Parser;
use std::cell::RefCell;

crate struct Driver<'tcx> {
    span_ctx: RefCell<span::Ctx>,
    arena: Arena<'tcx>,
    ir_arena: DroplessArena,
    global_ctx: OnceCell<GlobalCtx<'tcx>>,
    diagnostics: DiagnosticBuilder,
}

impl<'tcx> Driver<'tcx> {
    pub fn new(src: &str) -> Self {
        Self {
            span_ctx: RefCell::new(span::Ctx::new(src)),
            arena: Default::default(),
            ir_arena: Default::default(),
            global_ctx: OnceCell::new(),
            diagnostics: Default::default(),
        }
    }

    pub fn lex(&self) -> LResult<Vec<Tok>> {
        let mut span_ctx = self.span_ctx.borrow_mut();
        let mut lexer = Lexer::new(&mut span_ctx);
        let tokens = lexer.lex();
        Ok(tokens)
    }

    /// used for testing parsing
    crate fn parse_expr(&self) -> LResult<P<ast::Expr>> {
        let tokens = self.lex()?;
        let span_ctx = self.span_ctx.borrow();
        let mut parser = Parser::new(&span_ctx, tokens);
        let expr = parser.parse_expr()?;
        Ok(expr)
    }

    pub fn parse(&self) -> LResult<P<ast::Prog>> {
        let tokens = self.lex()?;
        let span_ctx = self.span_ctx.borrow();
        let mut parser = Parser::new(&span_ctx, tokens);
        let prog = parser.parse()?;
        Ok(prog)
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

    pub fn gen_tir(&'tcx self) -> LResult<tir::Prog<'tcx>> {
        let (ir, mut resolutions) = self.gen_ir()?;
        let defs = self.arena.alloc(std::mem::take(&mut resolutions.defs));
        let gcx = self.global_ctx.get_or_init(|| GlobalCtx::new(&self.arena, &defs));
        let tir = gcx.enter_tcx(|tcx| tcx.run_typeck(&ir));
        Ok(tir)
    }

    pub fn compile(&'tcx self) -> LResult<Executable> {
        let tir = self.gen_tir()?;
        println!("{}", tir);
        let gcx = self.global_ctx.get().unwrap();
        let ccx = gcx.enter_tcx(|tcx| self.arena.alloc(CompilerCtx::new(tcx)));
        let executable = Compilers::new(ccx).compile(&tir);
        Ok(executable)
    }

    pub fn exec(&'tcx self) -> LResult<exec::Val> {
        let executable = self.compile()?;
        let mut vm = VM::with_default_gc(executable);
        let value = vm.run()?;
        Ok(value)
    }
}
