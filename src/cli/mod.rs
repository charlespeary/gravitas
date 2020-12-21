use anyhow::Result;
use clap::Clap;

use crate::cli::commands::Subcommand;

pub mod commands;

pub type CommandOutput = Result<()>;

#[derive(Clap, Default, Debug, Clone)]
pub struct Settings {
    #[clap(short)]
    pub debug: bool,
    #[clap(subcommand)]
    pub subcmd: Subcommand,
}
