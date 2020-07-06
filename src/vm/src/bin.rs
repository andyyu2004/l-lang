use libvm;
use rustyline::error::ReadlineError;
use rustyline::Editor;

const HISTORY_PATH: &'static str = "repl_history";

fn main() {
    let mut rl = Editor::<()>::new();
    if rl.load_history(HISTORY_PATH).is_err() {}

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                if line.is_empty() {
                    continue;
                }
                rl.add_history_entry(line.as_str());
                if let Err(err) = libvm::exec(&line) {
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
