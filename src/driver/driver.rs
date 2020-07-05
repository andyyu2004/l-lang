use crate::ir;
use crate::{
    ast, compiler, ctx::Ctx, error::LResult, exec, lexer, parser, shared::Arena, tir, typeck::GlobalCtx
};
use compiler::Executable;
use exec::VM;
use ir::LoweringCtx;
use itertools::Itertools;
use lexer::Tok;
use once_cell::unsync::OnceCell;
use parser::Parser;

crate struct Driver<'tcx> {
    ctx: Ctx,
    arena: Arena<'tcx>,
    global_ctx: OnceCell<GlobalCtx<'tcx>>,
}

impl<'tcx> Driver<'tcx> {
    pub fn new(src: &str) -> Self {
        Self {
            ctx: Ctx::new(src),
            arena: Arena::default(),
            global_ctx: OnceCell::new(),
        }
    }

    pub fn lex(&self) -> LResult<Vec<Tok>> {
        let tokens = lexer::lex(&self.ctx.main_file().src).collect_vec();
        println!("{:#?}", tokens);
        Ok(tokens)
    }

    pub fn parse_expr(&self) -> LResult<ast::Expr> {
        let tokens = self.lex()?;
        let mut parser = Parser::new(&self.ctx, tokens);
        let expr = parser.parse_expr()?;
        println!("expr: {}", expr);
        Ok(expr)
    }

    pub fn gen_ir_expr(&self) -> LResult<&ir::Expr> {
        let expr = self.parse_expr()?;
        let mut lowering_ctx = LoweringCtx::new(&self.ctx.arena);
        let ir = lowering_ctx.lower_expr(&expr);
        println!("ir: {:?}", ir);
        Ok(ir)
    }

    pub fn gen_tir_expr(&'tcx self) -> LResult<&'tcx tir::Expr> {
        let ir = self.gen_ir_expr()?;
        let gcx = self.global_ctx.get_or_init(|| GlobalCtx::new(&self.arena));
        let tir = gcx.enter_tcx(|tcx| tcx.type_expr(ir)).unwrap();
        println!("tir: {}", tir);
        Ok(tir)
    }

    pub fn parse(&self) -> LResult<ast::Prog> {
        let tokens = self.lex()?;
        let mut parser = Parser::new(&self.ctx, tokens);
        let expr = parser.parse_expr()?;
        println!("expr: {}", expr);
        return Ok(ast::Prog { items: vec![] });
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
