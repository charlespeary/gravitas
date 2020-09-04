use clap::Clap;

#[derive(Clap, Default, Debug)]
pub struct VMSettings {
    /// Show executed opcodes
    #[clap(short)]
    pub debug: bool
}