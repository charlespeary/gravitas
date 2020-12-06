use anyhow::Result;
use clap::Clap;

pub use commands::Subcommand;

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
        Subcommand::Compile(compile) => {}
        Subcommand::Test(test) => {}
    }
    Ok(())
}
