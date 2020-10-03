use clap::App;
use libl;
use rustyline::error::ReadlineError;
use rustyline::Editor;

const HISTORY_PATH: &'static str = "repl_history";

fn main() {
    let mut rl = Editor::<()>::new();
    let yaml = clap::load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();
    let check = matches.is_present("check");
    let emit_tir = matches.is_present("tir");
    let emit_mir = matches.is_present("emit-mir");

    if let Some(path) = matches.value_of("INPUT") {
        let src = std::fs::read_to_string(path).unwrap();
        return if emit_mir {
            libl::dump_mir(&src).ok().unwrap_or_else(|| std::process::exit(1));
        } else if emit_tir {
            libl::dump_tir(&src).ok().unwrap_or_else(|| std::process::exit(1));
        } else if check {
            libl::check(&src).ok().unwrap_or_else(|| std::process::exit(1));
        } else {
            println!("{}", libl::llvm_exec(&src).unwrap_or_else(|_| std::process::exit(1)));
        };
    }

    if rl.load_history(HISTORY_PATH).is_err() {}

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                if line.is_empty() {
                    continue;
                }
                rl.add_history_entry(line.as_str());
                libl::llvm_exec_expr(&line).unwrap_or_else(|_| std::process::exit(1));
            }
            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    rl.save_history(HISTORY_PATH).unwrap();
}