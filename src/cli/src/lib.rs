#![feature(crate_visibility_modifier)]

mod subcommands;

use clap::Clap;
use ldriver::RunCfg;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clap)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Debug, Clap)]
enum SubCommand {
    Run(RunCfg),
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
        SubCommand::New(config) => subcommands::new(config),
        SubCommand::Run(config) => ldriver::run_compiler(config),
        SubCommand::Test(_) => todo!(),
    }
}
