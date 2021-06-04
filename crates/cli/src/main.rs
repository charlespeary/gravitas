use crate::options::Vtas;
use structopt::StructOpt;

pub(crate) mod options;
pub(crate) mod repl;

fn main() {
    let vtas = Vtas::from_args();

    match vtas {
        Vtas::Repl(repl) => repl.run(),
    }
}
