use clap::App;
use libvm;
use rustyline::error::ReadlineError;
use rustyline::Editor;

const HISTORY_PATH: &'static str = "repl_history";

fn main() {
    let mut rl = Editor::<()>::new();
    let yaml = clap::load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();

    if let Some(path) = matches.value_of("INPUT") {
        let src = std::fs::read_to_string(path).unwrap();
        // error reporting is in a kind of half ass state between `DiagnosticBuilder` and `LResult`
        return println!(
            "{}",
            libvm::exec(&src).unwrap_or_else(|err| {
                println!("{:?}", err);
                std::process::exit(1)
            })
        );
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
                if let Err(err) = libvm::exec_expr(&line) {
                    println!("{:?}", err);
                }
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
