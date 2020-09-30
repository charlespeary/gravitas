use clap::Clap;

#[derive(Clap, Default, Debug, Clone)]
pub struct Settings {
    /// Show executed opcodes
    #[clap(short)]
    pub debug: bool,
    /// Path to the file we want to interpret
    #[clap(short)]
    pub file_path: Option<String>,
}
