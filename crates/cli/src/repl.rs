use analyzer::analyze;
use bytecode::generate_bytecode;
use clap::Args;
use parser::parse;
use rustyline::{error::ReadlineError, Editor};

use crate::compiler::log_errors;
use vm::run;

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
                    let ast = parse(&code)
                        .map_err(|errors| log_errors(errors, &code))
                        .expect("Parsing failed. Investigate above errors to find the cause.");

                    analyze(&ast)
                        .map_err(|errors| log_errors(errors, &code))
                        .expect(
                            "Static analysis failed. Investigate above errors to find the cause.",
                        );

                    let bytecode = generate_bytecode(ast.clone())
                        .map_err(|error| println!("TODO: generation errors"))
                        .expect("Bytecode generation failed. Investigate above errors to find the cause.");

                    let program_output = run(bytecode, self.debug);
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
