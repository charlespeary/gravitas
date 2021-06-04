use crate::repl::Repl;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "vtas", about = "A toy programming language")]
pub(crate) enum Vtas {
    Repl(Repl),
}
