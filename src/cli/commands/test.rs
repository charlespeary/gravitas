use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use clap::Clap;
use walkdir::WalkDir;

static TEST_EXTENSION: &str = "vtest";

#[derive(Clap, Default, Debug, Clone)]
pub struct Test {
    #[clap(short)]
    pub path: Option<String>,
}

fn is_test_file<'a>(
    entry: std::result::Result<walkdir::DirEntry, walkdir::Error>,
) -> Option<PathBuf> {
    entry
        .ok()
        .map(|e| {
            if e.path()
                .extension()
                .map(|ext| ext == TEST_EXTENSION)
                .unwrap_or(false)
            {
                Some(e.into_path())
            } else {
                None
            }
        })
        .flatten()
}

pub fn run_test(settings: Test) -> Result<()> {
    // Search for files in path supplied by user or the one where script is run
    let path = settings.path.as_deref().unwrap_or(".");
    let test_files: Vec<PathBuf> = WalkDir::new(path)
        .into_iter()
        .filter_map(is_test_file)
        .collect();

    for path in test_files {
        let code = fs::read_to_string(path)?;
        println!("{}", code);
    }

    Ok(())
}
