use crate::repl::Repl;
use clap::{Parser, Subcommand};

#[derive(Parser)]
pub(crate) struct Gravitas {
    #[command(subcommand)]
    pub(crate) action: GravitasAction,
}

#[derive(Subcommand)]
pub(crate) enum GravitasAction {
    Repl(Repl),
}
