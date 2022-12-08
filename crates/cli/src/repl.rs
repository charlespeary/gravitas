use analyzer::analyze;
use lasso::IntoReader;
use parser::{parse, parse::Ast};
use rustyline::{error::ReadlineError, Editor};
use structopt::StructOpt;

use crate::compiler::log_errors;
use vm::VM;

#[derive(Debug, StructOpt)]
pub(crate) struct Repl {
    #[structopt(short, long)]
    pub(crate) debug: bool,
}

impl Repl {
    pub(crate) fn run(&self) {
        let mut rl = Editor::<()>::new();
        let mut ast: Ast = vec![];

        loop {
            let readline = rl.readline(">> ");
            match readline {
                Ok(code) => {
                    rl.add_history_entry(code.as_str());
                    let (mut new_ast, symbols) = parse(&code)
                        .map_err(|(errors, symbols)| log_errors(errors, symbols, &code))
                        .expect("Parsing failed. Investigate above errors to find the cause.");

                    ast.append(&mut new_ast);

                    analyze(&ast)
                        .map_err(|errors| log_errors(errors, symbols, &code))
                        .expect(
                            "Static analysis failed. Investigate above errors to find the cause.",
                        );
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
