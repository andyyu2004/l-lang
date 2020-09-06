use clap::App;
use libvm;
use rustyline::error::ReadlineError;
use rustyline::Editor;

const HISTORY_PATH: &'static str = "repl_history";

fn main() {
    let mut rl = Editor::<()>::new();
    let yaml = clap::load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();
    let interpret = matches.is_present("interpret");

    if let Some(path) = matches.value_of("INPUT") {
        let src = std::fs::read_to_string(path).unwrap();
        return if interpret {
            unimplemented!();
        } else {
            println!("{}", libvm::llvm_exec(&src).unwrap_or_else(|_| std::process::exit(1)));
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
                libvm::llvm_exec_expr(&line).unwrap_or_else(|_| std::process::exit(1));
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
