use clap::Clap;

pub use crate::cli::commands::{compile::Compile, test::Test};

pub mod compile;
pub mod test;

#[derive(Clap, Debug, Clone)]
pub enum Subcommand {
    Test(Test),
    Compile(Compile),
}

impl Default for Subcommand {
    fn default() -> Self {
        Subcommand::Compile(Compile {
            file_path: String::from("main.rlox"),
            debug: false,
        })
    }
}
