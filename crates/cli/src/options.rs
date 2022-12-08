use crate::repl::Repl;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "gravitas", about = "A toy programming language")]
pub(crate) enum Gravitas {
    Repl(Repl),
}
