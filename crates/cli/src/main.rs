use crate::options::Gravitas;
use structopt::StructOpt;

pub(crate) mod compiler;
pub(crate) mod options;
pub(crate) mod repl;

fn main() {
    let gravitas = Gravitas::from_args();

    match gravitas {
        Gravitas::Repl(repl) => repl.run(),
    }
}
