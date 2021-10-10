#![feature(crate_visibility_modifier)]
#![feature(process_exitcode_placeholder)]

mod subcommands;

use clap::{AppSettings, Clap};
use lc_driver::CompilerOptions;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clap)]
#[clap(setting(AppSettings::InferSubcommands))]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Debug, Clap)]
enum SubCommand {
    Jit(CompilerOptions),
    Build(CompilerOptions),
    Run(CompilerOptions),
    Check(CompilerOptions),
    New(NewCmd),
    Test(TestCmd),
}

#[derive(Debug, Clap)]
struct NewCmd {
    path: PathBuf,
}

#[derive(Debug, Clap)]
struct TestCmd {}

pub fn main() -> io::Result<()> {
    let opts = Opts::parse();
    match opts.subcmd {
        SubCommand::New(ncfg) => subcommands::new(ncfg),
        // TODO the interface needs some work
        SubCommand::Jit(rcfg) => {
            let _ = lc_driver::run_compiler(rcfg, |compiler| compiler.llvm_jit());
            Ok(())
        }
        SubCommand::Run(rcfg) => {
            let _ = lc_driver::run_compiler(rcfg, |compiler| compiler.run());
            Ok(())
        }
        SubCommand::Build(bcfg) => {
            let _ = lc_driver::run_compiler(bcfg, |compiler| compiler.build());
            Ok(())
        }
        SubCommand::Check(cfg) => {
            let _ = lc_driver::run_compiler(cfg, |compiler| compiler.check());
            Ok(())
        }
        SubCommand::Test(_) => todo!(),
    }
}
