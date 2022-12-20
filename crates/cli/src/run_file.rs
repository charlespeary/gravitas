use std::{fs::read_to_string, path::Path};

use clap::Args;

use vm::run;

use crate::compiler::compile_and_run;

#[derive(Debug, Args)]
pub(crate) struct RunFile {
    #[clap(long, short, action)]
    pub(crate) debug: bool,
    #[arg(short, long)]
    file_path: String,
}

impl RunFile {
    pub(crate) fn run(&self) {
        let path = Path::new(&self.file_path);
        let code = read_to_string(path).expect("File not found!");
        compile_and_run(&code, self.debug);
    }
}
