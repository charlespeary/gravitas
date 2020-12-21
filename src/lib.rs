extern crate derive_more;
#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

use anyhow::Result;
use clap::Clap;

pub use crate::{
    bytecode::BytecodeGenerator,
    cli::{commands::exec_commands, Settings},
    parser::Parser,
    vm::VM,
};

mod bytecode;
mod cli;
mod compiler;
mod parser;
mod std;
mod utils;
mod vm;

pub fn run() -> Result<()> {
    let settings: Settings = Settings::parse();

    exec_commands(settings);

    Ok(())
}
