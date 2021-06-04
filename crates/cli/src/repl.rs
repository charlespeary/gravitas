use parser::parse;
use rustyline::{error::ReadlineError, Editor};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub(crate) struct Repl {
    #[structopt(short, long)]
    pub(crate) debug: bool,
}

impl Repl {
    pub(crate) fn run(&self) {
        let mut rl = Editor::<()>::new();

        loop {
            let readline = rl.readline(">> ");
            match readline {
                Ok(line) => {
                    rl.add_history_entry(line.as_str());
                    let program = parse(&line);
                }
                Err(ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break;
                }
                Err(ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
    }
}
