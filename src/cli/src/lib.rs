#![feature(crate_visibility_modifier)]
#![feature(process_exitcode_placeholder)]

mod subcommands;

use clap::Clap;
use ldriver::CompilerOptions;
use std::io;
use std::path::PathBuf;
use std::process::ExitCode;

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

pub fn main() -> io::Result<ExitCode> {
    let opts = Opts::parse();
    match opts.subcmd {
        SubCommand::New(ncfg) => subcommands::new(ncfg),
        // TODO actually run the built executable
        SubCommand::Run(rcfg) => ldriver::run_compiler(rcfg),
        SubCommand::Build(bcfg) => ldriver::run_compiler(bcfg),
        SubCommand::Check(cfg) => ldriver::run_compiler(cfg),
        SubCommand::Test(_) => todo!(),
    }
}
