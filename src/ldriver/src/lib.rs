#![feature(crate_visibility_modifier)]
#![feature(never_type)]
#![feature(once_cell)]
#![feature(decl_macro)]

mod cli_error;
mod config;
mod passes;
mod queries;

#[macro_use]
extern crate colour;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate log;

use ast::{ExprKind, P};
use astlowering::AstLoweringCtx;
use clap::App;
use cli_error::{CliError, CliResult};
use codegen::CodegenCtx;
use config::LConfig;
use error::{LError, LResult};
use inkwell::context::Context as LLVMCtx;
use inkwell::values::FunctionValue;
use inkwell::OptimizationLevel;
use lcore::{GlobalCtx, TyCtx};
use lex::{Lexer, Tok};
use log::LevelFilter;
use mir::dump_mir;
use parse::Parser;
use resolve::Resolutions;
use resolve::Resolver;
use resolve::ResolverArenas;
use session::Session;
use span::{SourceMap, ROOT_FILE_IDX, SPAN_GLOBALS};
use std::env::temp_dir;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io;
use std::io::Write;
use std::lazy::OnceCell;
use std::path::Path;
use std::rc::Rc;

pub fn main() -> io::Result<()> {
    let _ = std::fs::remove_file("log.txt");
    let level_filter = if cfg!(debug_assertions) { LevelFilter::Trace } else { LevelFilter::Info };
    simple_logging::log_to_file("log.txt", level_filter).unwrap();

    let yaml = clap::load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();
    let _check = matches.is_present("check");
    let _emit_tir = matches.is_present("tir");
    let _emit_mir = matches.is_present("emit-mir");
    // TODO take optimization level as parameter

    let path_str = matches.value_of("INPUT").unwrap();
    let path = Path::new(path_str);

    let config = config::load_config(path)?;

    let driver = Driver::new(config);
    match driver.llvm_exec() {
        Ok(i) => std::process::exit(i),
        Err(..) => std::process::exit(1),
    }

    // if let Some(path) = matches.value_of("INPUT") {
    //     let src = std::fs::read_to_string(path).unwrap();
    //     if emit_mir {
    //         driver.dump_mir(&src).ok().unwrap_or_else(|| std::process::exit(1));
    //     } else if emit_tir {
    //         libl::dump_tir(&src).ok().unwrap_or_else(|| std::process::exit(1));
    //     } else if check {
    //         libl::check(&src).ok().unwrap_or_else(|| std::process::exit(1));
    //     } else {
    //         println!("{}", libl::llvm_exec(&src).unwrap_or_else(|_| std::process::exit(1)));
    //     };
}

pub struct Driver<'tcx> {
    sess: Session,
    core_arenas: lcore::Arena<'tcx>,
    ir_arena: astlowering::Arena<'tcx>,
    resolver_arenas: ResolverArenas<'tcx>,
    global_ctx: OnceCell<GlobalCtx<'tcx>>,
    llvm_ctx: LLVMCtx,
}

/// exits if any errors have been reported
macro check_errors($self:expr, $ret:expr) {{
    if $self.sess.has_errors() {
        let errc = $self.sess.err_count();
        e_red_ln!("{} error{} emitted", errc, util::pluralize!(errc));
        Err(LError::ErrorReported)
    } else {
        Ok($ret)
    }
}}

impl<'tcx> Driver<'tcx> {
    /// creates a temporary file and proceeds as usual
    pub fn from_src(src: &str) -> Self {
        let mut path = temp_dir();
        let mut hasher = std::collections::hash_map::DefaultHasher::default();
        src.hash(&mut hasher);
        // an attempt at making the file name sufficiently unique
        // to avoid the parallel tests overwriting each other's files and becoming a mess
        let hash = hasher.finish();
        path.push(format!("tmp{}.l", hash));
        let mut file = File::create(&path).unwrap();
        file.write(src.as_bytes()).unwrap();
        Self::new(LConfig::from_main_path(path))
    }

    pub fn new(config: LConfig) -> Self {
        SPAN_GLOBALS
            .with(|globals| *globals.source_map.borrow_mut() = SourceMap::new(config.main_path));

        Self {
            llvm_ctx: LLVMCtx::create(),
            resolver_arenas: Default::default(),
            core_arenas: Default::default(),
            ir_arena: Default::default(),
            global_ctx: Default::default(),
            sess: Default::default(),
        }
    }

    pub fn parse(&self) -> LResult<P<ast::Ast>> {
        // assume one file for now
        let mut parser = Parser::new(&self.sess, ROOT_FILE_IDX);
        let ast = parser.parse();
        // error!("{:#?}", ast);
        check_errors!(self, ast.unwrap())
    }

    pub fn gen_ir(&'tcx self) -> LResult<(&'tcx ir::Ir<'tcx>, Resolutions)> {
        let ast = self.parse()?;
        let mut resolver = Resolver::new(&self.sess, &self.resolver_arenas);
        resolver.resolve(&ast);
        let lctx = AstLoweringCtx::new(&self.ir_arena, &self.sess, &mut resolver);
        let ir = lctx.lower_prog(&ast);
        let resolutions = resolver.complete();
        debug!("{:#?}", ir);
        Ok((ir, resolutions))
    }

    fn with_tcx<R>(&'tcx self, f: impl FnOnce(TyCtx<'tcx>) -> R) -> LResult<R> {
        let (ir, resolutions) = self.gen_ir()?;
        let gcx = self.global_ctx.get_or_init(|| {
            GlobalCtx::new(ir, &self.core_arenas, resolutions, &self.sess, queries::query_ctx())
        });
        gcx.enter_tcx(|tcx| tcx.analyze(()));
        let ret = gcx.enter_tcx(|tcx| f(tcx));
        check_errors!(self, ret)
    }

    pub fn dump_mir(&'tcx self) -> LResult<()> {
        self.with_tcx(|tcx| dump_mir(tcx, &mut std::io::stderr()))
    }

    pub fn check(&'tcx self) -> LResult<()> {
        self.with_tcx(|tcx| tcx.analyze(()))
    }

    pub fn create_codegen_ctx(&'tcx self) -> LResult<CodegenCtx> {
        self.with_tcx(|tcx| CodegenCtx::new(tcx, &self.llvm_ctx))
    }

    pub fn llvm_compile(&'tcx self) -> LResult<(CodegenCtx, FunctionValue<'tcx>)> {
        let mut cctx = self.create_codegen_ctx()?;
        let main_fn = cctx.codegen();
        check_errors!(self, (cctx, main_fn.unwrap()))
    }

    pub fn llvm_exec(&'tcx self) -> LResult<i32> {
        let (cctx, main_fn) = self.llvm_compile()?;
        println!("---");
        let jit = cctx.module.create_jit_execution_engine(OptimizationLevel::None).unwrap();
        let val = unsafe { jit.run_function_as_main(main_fn, &[]) };
        Ok(val)
    }

    pub fn has_errors(&self) -> bool {
        self.sess.has_errors()
    }

    pub fn lex(&self) -> LResult<Vec<Tok>> {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex(ROOT_FILE_IDX);
        Ok(tokens)
    }

    /// used for testing parsing
    pub fn parse_expr(&self) -> Option<P<ast::Expr>> {
        let mut parser = Parser::new(&self.sess, ROOT_FILE_IDX);
        let expr = parser.parse_expr();
        match &expr.kind {
            ExprKind::Err => None,
            _ => Some(expr),
        }
    }
}

impl<'tcx> Driver<'tcx> {
    pub fn gen_tir(&'tcx self) -> LResult<tir::Prog<'tcx>> {
        self.with_tcx(mir::build_tir)?
    }
}
