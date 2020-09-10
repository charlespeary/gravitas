use std::fs::read_to_string;
use std::io::stdin;

use anyhow::{Context, Result};

use crate::parser::compile;
use crate::settings::Settings;

pub use self::iter::{peek_nth, PeekNth};

mod iter;

pub fn initialize(settings: &Settings) -> Result<()> {
    if let Some(path) = &settings.file_path {
        let code = read_to_string(path).with_context(|| "Given input file doesn't exist.")?;
        compile(&code);
    } else {
        loop {
            println!("> ");
            use std::io;

            let mut input = String::new();
            match stdin().read_line(&mut input) {
                Ok(n) => compile(&input),
                Err(error) => println!("error: {}", error),
            }
        }
    }
    Ok(())
}
