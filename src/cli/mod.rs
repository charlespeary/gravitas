use anyhow::Result;
use clap::Clap;

use crate::cli::commands::{compile::run_compile, test::run_test, Subcommand};

pub mod commands;

#[derive(Clap, Default, Debug, Clone)]
pub struct Settings {
    /// Show executed opcodes
    #[clap(short)]
    pub debug: bool,
    #[clap(subcommand)]
    pub subcmd: Subcommand,
}

pub fn exec_commands(settings: Settings) -> Result<()> {
    match settings.subcmd {
        Subcommand::Compile(compile) => {
            run_compile(compile);
        }
        Subcommand::Test(test) => {
            run_test(test);
        }
    }
    Ok(())
}
