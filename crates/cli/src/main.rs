use clap::Parser;
use options::GravitasAction;

use crate::options::Gravitas;

pub(crate) mod compiler;
pub(crate) mod options;
pub(crate) mod repl;

fn main() {
    let gravitas = Gravitas::parse();

    match gravitas.action {
        GravitasAction::Repl(repl) => repl.run(),
    }
}
