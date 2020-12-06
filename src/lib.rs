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
    cli::{exec_commands, Settings, Subcommand},
    parser::Parser,
    vm::VM,
};

mod bytecode;
mod cli;
mod parser;
mod std;
mod utils;
mod vm;

pub fn run() -> Result<()> {
    let settings: Settings = Settings::parse();

    exec_commands(settings)?;

    Ok(())
}
