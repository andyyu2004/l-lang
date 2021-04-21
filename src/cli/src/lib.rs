#![feature(crate_visibility_modifier)]
#![feature(process_exitcode_placeholder)]

mod subcommands;

use clap::Clap;
use ldriver::CompilerOptions;
use std::io;
use std::path::PathBuf;


#[derive(Debug, Clap)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Debug, Clap)]
enum SubCommand {
    Run(CompilerOptions),
    Build(CompilerOptions),
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
        SubCommand::Run(rcfg) => {
            let _ = ldriver::run_compiler(rcfg, |compiler| compiler.llvm_jit());
            Ok(())
        }
        SubCommand::Build(bcfg) => {
            let _ = ldriver::run_compiler(bcfg, |compiler| compiler.build());
            Ok(())
        }
        SubCommand::Check(cfg) => {
            let _ = ldriver::run_compiler(cfg, |compiler| compiler.check());
            Ok(())
        }
        SubCommand::Test(_) => todo!(),
    }
}
