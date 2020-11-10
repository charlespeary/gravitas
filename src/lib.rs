extern crate derive_more;
#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

use anyhow::Result;
use clap::Clap;

use settings::Settings;
use utils::initialize;

pub use crate::{bytecode::BytecodeGenerator, parser::Parser, vm::VM};

mod bytecode;
mod parser;
mod settings;
mod std;
mod utils;
mod vm;

pub fn run() -> Result<()> {
    let settings = Settings::parse();
    match initialize(settings) {
        Ok(_) => {}
        Err(e) => {
            utils::log::title_error("ERROR");
            utils::log::body(&e);
        }
    }
    Ok(())
}
