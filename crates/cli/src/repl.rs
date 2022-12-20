use clap::Args;
use rustyline::{error::ReadlineError, Editor};

use crate::compiler::compile_and_run;

#[derive(Debug, Args)]
pub(crate) struct Repl {
    #[clap(long, short, action)]
    pub(crate) debug: bool,
}

impl Repl {
    pub(crate) fn run(&self) {
        let mut rl = Editor::<()>::new();

        loop {
            let readline = rl.readline(">> ");
            match readline {
                Ok(code) => {
                    rl.add_history_entry(code.as_str());
                    let program_output = compile_and_run(&code, self.debug);

                    println!("> {}", program_output);
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
