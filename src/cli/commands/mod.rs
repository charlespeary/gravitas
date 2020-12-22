use clap::Clap;

pub use crate::{
    cli::commands::{
        compile::{run_compile, Compile},
        test::{run_test, Test},
    },
    Settings,
};

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
        })
    }
}

pub fn exec_commands(settings: Settings) {
    match &settings.subcmd {
        Subcommand::Compile(compile) => {
            run_compile(&settings, compile);
        }
        Subcommand::Test(test) => {
            run_test(&settings, test);
        }
    };
}
