use clap::Clap;

mod compile;
mod test;

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

#[derive(Clap, Default, Debug, Clone)]
pub struct Test {}

#[derive(Clap, Default, Debug, Clone)]
pub struct Compile {
    /// Path to the file we want to interpret
    #[clap(short, default_value = "main.rlox")]
    pub file_path: String,
    #[clap(short)]
    pub debug: bool,
}
