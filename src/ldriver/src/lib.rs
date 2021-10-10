#![feature(crate_visibility_modifier)]
#![feature(process_exitcode_placeholder)]
#![feature(box_syntax)]
#![feature(never_type)]
#![feature(panic_info_message)]
#![feature(once_cell)]
#![feature(decl_macro)]

mod cli_error;
mod config;
mod passes;
mod queries;

#[macro_use]
extern crate log;

#[macro_use]
extern crate colour;

#[macro_use]
extern crate serde;

use ast::{ExprKind, P};
use astlowering::AstLoweringCtx;
use codegen::CodegenCtx;
use codespan_reporting::diagnostic::Diagnostic;
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;
use config::LConfig;
use error::{ErrorFormat, ErrorReported, LResult};
use index::IndexVec;
use inkwell::context::Context as LLVMCtx;

use inkwell::OptimizationLevel;
use ir::{PkgId, Resolutions};
use lcore::{GlobalCtx, TyCtx};
use lex::{Lexer, TokenIterator, TokenStream};
use log::LevelFilter;
use meta::PkgMetadata;
use parse::Parser;
use resolve::{Resolver, ResolverArenas};
pub use session::{CompilerOptions, Session};
use span::{sym, SourceMap, ROOT_FILE_IDX, SPAN_GLOBALS};
use std::fs::File;
use std::io::Write;
use std::lazy::OnceCell;
use std::path::PathBuf;
use termcolor::{BufferedStandardStream, ColorChoice};

lazy_static::lazy_static! {
    /// just a dummy to allow us to use codespan to print out our panics
    static ref SIMPLE_FILES: SimpleFiles<&'static str, &'static str> = SimpleFiles::new();
}

pub fn run_compiler<R>(
    opts: CompilerOptions,
    f: impl for<'tcx> FnOnce(&'tcx Driver<'tcx>) -> R,
) -> R {
    // our error handling in here is basically just using panic!()
    // this makes the output look nicer and consistent with the compiler errors
    std::panic::set_hook(box move |info| {
        if let Some(msg) = info.message() {
            let mut buf = String::new();
            std::fmt::write(&mut buf, *msg).unwrap();
            let diag = Diagnostic::error().with_message(buf);
            // nothing gets printed if we construct this stream in lazy_static!
            let mut writer = BufferedStandardStream::stdout(ColorChoice::Auto);
            term::emit(&mut writer, &term::Config::default(), &*SIMPLE_FILES, &diag).unwrap();
            writer.flush().unwrap();
            std::process::exit(1);
        }
    });

    let _ = std::fs::remove_file("l.log");
    let level_filter = if cfg!(debug_assertions) { LevelFilter::Trace } else { LevelFilter::Info };
    simple_logging::log_to_file("l.log", level_filter).unwrap();

    let lconfig = config::load_config(opts).unwrap_or_else(|err| panic!("{}", err));

    // we unregister our panic hook above as the "panic error handling" section is over
    let _ = std::panic::take_hook();

    f(&Driver::new(lconfig))
}

pub fn compile(lconfig: LConfig) -> i32 {
    let driver = Driver::new(lconfig);
    match driver.llvm_jit() {
        Ok(_) => 0,
        Err(..) => 1,
    }
}

pub struct Driver<'tcx> {
    sess: Session,
    root_path: PathBuf,
    /// metadata of the dependencies specified in `L.toml`
    dependencies: IndexVec<PkgId, PkgMetadata>,
    core_arenas: lcore::Arena<'tcx>,
    ir_arena: astlowering::Arena<'tcx>,
    resolver_arenas: ResolverArenas<'tcx>,
    global_ctx: OnceCell<GlobalCtx<'tcx>>,
    llvm_ctx: LLVMCtx,
}

/// exits if any errors have been reported
macro_rules! check_errors {
    ($self:expr) => {{ check_errors!($self, ())? }};
    ($self:expr, $ret:expr) => {{
        if $self.sess.has_errors() {
            // don't print out this info for other formats as parsing the error output will become hard
            if $self.sess.opts.error_format == ErrorFormat::Text {
                let errc = $self.sess.err_count();
                let warnings = $self.sess.warning_count();
                if warnings > 0 {
                    e_yellow_ln!("{} warning{} emitted", warnings, lutil::pluralize!(warnings));
                }
                e_red_ln!("{} error{} emitted", errc, lutil::pluralize!(errc));
            }
            Err(ErrorReported)
        } else {
            Ok($ret)
        }
    }}
}

impl<'tcx> Driver<'tcx> {
    /// creates a temporary file and proceeds as usual
    // this can't be made #[cfg(test)] for some reason
    // as some test code complains this doesn't exist
    pub fn from_src(src: &str) -> Self {
        let tempdir = tempfile::tempdir().unwrap();
        // into_place ensure the tempdir is *not* dropped
        // we need it later in the `run` stage
        let main_path = tempdir.into_path().join("main.l");
        let mut file = File::create(&main_path).unwrap();
        file.write(src.as_bytes()).unwrap();
        Self::new(LConfig::from_main_path(main_path))
    }

    pub fn new(config: LConfig) -> Self {
        let path = config.root_path.join(&config.bin.main_path);
        SPAN_GLOBALS.with(|globals| *globals.source_map.borrow_mut() = SourceMap::new(&path));

        // let dependencies = config.dependencies.values().collect();
        let dependencies = IndexVec::new();

        Self {
            dependencies,
            llvm_ctx: LLVMCtx::create(),
            root_path: config.root_path,
            sess: Session::create(config.opts),
            resolver_arenas: Default::default(),
            core_arenas: Default::default(),
            ir_arena: Default::default(),
            global_ctx: Default::default(),
        }
    }

    pub fn parse(&self) -> LResult<P<ast::Ast>> {
        // assume one file for now
        let mut parser = Parser::new(&self.sess);
        let ast = parser.parse();
        // error!("{:#?}", ast);
        check_errors!(self, ast.unwrap())
    }

    pub fn gen_ir(&'tcx self) -> LResult<(&'tcx ir::Ir<'tcx>, Resolutions)> {
        let ast = self.parse()?;
        let mut resolver = Resolver::new(&self.sess, &self.resolver_arenas, &self.dependencies);
        resolver.resolve(&ast);
        let lctx = AstLoweringCtx::new(&self.ir_arena, &self.sess, &mut resolver);
        let ir = lctx.lower_ast(&ast);
        let resolutions = resolver.complete();
        Ok((ir, resolutions))
    }

    pub fn with_tcx<R>(&'tcx self, f: impl FnOnce(TyCtx<'tcx>) -> R) -> LResult<R> {
        let (ir, resolutions) = self.gen_ir()?;
        let gcx = self.global_ctx.get_or_init(|| {
            GlobalCtx::new(ir, &self.core_arenas, resolutions, &self.sess, queries::query_ctx())
        });
        gcx.enter_tcx(|tcx| tcx.analyze(()));
        let ret = gcx.enter_tcx(f);
        check_errors!(self, ret)
    }

    pub fn check(&'tcx self) -> LResult<()> {
        self.with_tcx(|tcx| tcx.analyze(()))
    }

    pub fn create_codegen_ctx(&'tcx self) -> LResult<CodegenCtx<'tcx>> {
        self.with_tcx(|tcx| CodegenCtx::new(tcx, &self.llvm_ctx))
    }

    pub fn build(&'tcx self) -> LResult<()> {
        self.llvm_compile()?;
        Ok(())
    }

    pub fn llvm_compile(&'tcx self) -> LResult<CodegenCtx<'tcx>> {
        let cctx = self.create_codegen_ctx()?;
        cctx.codegen()?;
        check_errors!(self);
        let ir_path = self.root_path.join("llvm-ir.ll");
        cctx.module.print_to_file(&ir_path).unwrap_or_else(|err| panic!("{}", err));
        // let bitcode_path = self.root_path.join("build.bc");
        // assert!(cctx.module.write_bitcode_to_path(&bitcode_path));
        let output_path = self.root_path.join("l.out");
        std::process::Command::new("clang")
            .arg(&ir_path)
            .arg("-o")
            .arg(output_path)
            .arg("-lgc")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .expect("failed to link using clang (is `clang` on your path?)");
        Ok(cctx)
    }

    pub fn run(&'tcx self) -> LResult<Option<i32>> {
        self.build()?;
        let path = self.root_path.join("l.out");
        assert!(path.exists());
        Ok(std::process::Command::new(path).status().expect("io error").code())
    }

    // TODO does not link to libgc so will segfault if run with anything that uses `box`
    pub fn llvm_jit(&'tcx self) -> LResult<i32> {
        let cctx = self.llvm_compile()?;
        let jit = cctx.module.create_jit_execution_engine(OptimizationLevel::None).unwrap();
        let main = cctx.module.get_function(sym::main.as_str()).unwrap();
        let val = unsafe { jit.run_function_as_main(main, &[]) };
        Ok(val)
    }

    pub fn has_errors(&self) -> bool {
        self.sess.has_errors()
    }

    pub fn lex(&self) -> LResult<TokenIterator> {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex(ROOT_FILE_IDX);
        Ok(tokens)
    }

    pub fn parse_tt(&self) -> Result<TokenStream, TokenStream> {
        let mut parser = Parser::new(&self.sess);
        let tt = parser.test_parse_tt();
        if self.has_errors() { Err(tt) } else { Ok(tt) }
    }

    pub fn parse_expr(&self) -> Option<P<ast::Expr>> {
        let mut parser = Parser::new(&self.sess);
        let expr = parser.test_parse_expr();
        match &expr.kind {
            ExprKind::Err => None,
            _ => Some(expr),
        }
    }

    pub fn parse_macro(&self) -> Option<ast::Macro> {
        let mut parser = Parser::new(&self.sess);
        match parser.test_parse_macro() {
            Ok(mac) => Some(mac),
            Err(err) => {
                err.emit();
                None
            }
        }
    }
}

impl<'tcx> Driver<'tcx> {
    pub fn gen_tir(&'tcx self) -> LResult<tir::Prog<'tcx>> {
        self.with_tcx(mirgen::build_tir)?
    }
}
