use anyhow::{Error, Result};
use clap::Clap;

use crate::{
    bytecode::BytecodeGenerator,
    cli::CommandOutput,
    compiler::compile_path,
    parser::{Parser, Token},
    Settings,
    utils::Either,
    utils::log,
    vm::VM,
};

#[derive(Clap, Default, Debug, Clone)]
pub struct Compile {
    /// Path to the file we want to interpret
    #[clap(short, default_value = "main.vtas")]
    pub file_path: String,
}

type Compiled = Result<(), Either<Error, Vec<Error>>>;

pub fn run_compile(global_settings: &Settings, cmd_settings: &Compile) -> CommandOutput {
    // Pretty print the compilation errors
    let program =
        compile_path(&cmd_settings.file_path, global_settings).expect("Compilation error");
    let mut vm = VM::from(global_settings.clone());
    let result = vm.interpret(program);

    Ok(())
}
