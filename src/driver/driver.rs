use crate::ir;
use crate::{ast, compiler, ctx::Ctx, error::LResult, exec, lexer, parser, tir};
use compiler::Executable;
use exec::VM;
use itertools::Itertools;
use lexer::Tok;
use parser::Parser;

crate struct Driver {
    ctx: Ctx,
}

impl Driver {
    pub fn new(src: &str) -> Self {
        Self { ctx: Ctx::new(src) }
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
