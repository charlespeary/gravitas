use anyhow::{Error, Result};
use clap::Clap;

use crate::{
    bytecode::BytecodeGenerator,
    cli::CommandOutput,
    compiler::compile_path,
    parser::{Parser, Token},
    Settings,
    utils::{Either, logger},
    vm::VM,
};
use crate::vm::utilities::Utilities;

#[derive(Clap, Default, Debug, Clone)]
pub struct Compile {
    /// Path to the file we want to interpret
    #[clap(short, default_value = "main.vt")]
    pub path: String,
}

type Compiled = Result<(), Either<Error, Vec<Error>>>;

pub fn run_compile(global_settings: &Settings, cmd_settings: &Compile) -> CommandOutput {
    // Pretty print the compilation errors
    let program =
        compile_path(&cmd_settings.path, global_settings).expect("Compilation error");
    let mut utilities = Utilities::default().with_settings(global_settings);
    let mut vm = VM::default().with_utilities(&mut utilities);
    let result = vm.interpret(program);

    Ok(())
}
