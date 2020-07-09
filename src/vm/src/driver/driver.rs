use crate::ast::{self, P};
use crate::compiler::Executable;
use crate::error::{LError, LResult};
use crate::shared::Arena;
use crate::typeck::GlobalCtx;
use crate::{arena::DroplessArena, exec, ir, lexer, parser, span, tir};
use exec::VM;
use ir::LoweringCtx;
use lexer::{Lexer, Tok};
use once_cell::unsync::OnceCell;
use parser::Parser;
use std::cell::RefCell;

crate struct Driver<'tcx> {
    span_ctx: RefCell<span::Ctx>,
    arena: Arena<'tcx>,
    ir_arena: DroplessArena,
    global_ctx: OnceCell<GlobalCtx<'tcx>>,
}

impl<'tcx> Driver<'tcx> {
    pub fn new(src: &str) -> Self {
        Self {
            span_ctx: RefCell::new(span::Ctx::new(src)),
            arena: Default::default(),
            ir_arena: Default::default(),
            global_ctx: OnceCell::new(),
        }
    }

    pub fn lex(&self) -> LResult<Vec<Tok>> {
        let mut span_ctx = self.span_ctx.borrow_mut();
        let mut lexer = Lexer::new(&mut span_ctx);
        let tokens = lexer.lex();
        Ok(tokens)
    }

    pub fn parse_expr(&self) -> LResult<P<ast::Expr>> {
        let tokens = self.lex()?;
        let span_ctx = self.span_ctx.borrow();
        let mut parser = Parser::new(&span_ctx, tokens);
        let expr = parser.parse_expr()?;
        println!("expr: {}", expr);
        Ok(expr)
    }

    pub fn gen_ir_expr(&self) -> LResult<&ir::Expr> {
        let expr = self.parse_expr()?;
        let mut lowering_ctx = LoweringCtx::new(&self.ir_arena);
        let ir = lowering_ctx.lower_expr(&expr);
        println!("ir: {:?}", ir);
        Ok(ir)
    }

    pub fn gen_tir_expr(&'tcx self) -> LResult<&'tcx tir::Expr> {
        let ir = self.gen_ir_expr()?;
        let gcx = self.global_ctx.get_or_init(|| GlobalCtx::new(&self.arena));
        let tir = gcx
            .enter_tcx(|tcx| tcx.type_expr(ir))
            .map_err(|err| LError::Error(format!("{}", err)))?;
        println!("tir: {}", tir);
        Ok(tir)
    }

    pub fn parse(&self) -> LResult<P<ast::Prog>> {
        let tokens = self.lex()?;
        let span_ctx = self.span_ctx.borrow();
        let mut parser = Parser::new(&span_ctx, tokens);
        let prog = parser.parse()?;
        return Ok(prog);
    }

    pub fn gen_ir(&self) -> LResult<ir::Prog> {
        let ast = self.parse();
        todo!()
    }

    pub fn gen_tir(&self) -> LResult<tir::Prog> {
        let ir = self.gen_ir();
        todo!()
    }

    pub fn compile(&self) -> LResult<Executable> {
        let tir = self.gen_tir();
        todo!()
    }

    pub fn exec(&self) -> LResult<exec::Val> {
        let executable = self.compile()?;
        let mut vm = VM::with_default_gc(executable);
        let value = vm.run()?;
        Ok(value)
    }
}
