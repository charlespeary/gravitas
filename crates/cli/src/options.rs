use crate::{repl::Repl, run_file::RunFile};
use clap::{Parser, Subcommand};

#[derive(Parser)]
pub(crate) struct Gravitas {
    #[command(subcommand)]
    pub(crate) action: GravitasAction,
}

#[derive(Subcommand)]
pub(crate) enum GravitasAction {
    Repl(Repl),
    RunFile(RunFile),
}
