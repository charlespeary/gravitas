use std::path::PathBuf;

use clap::Clap;
use walkdir::WalkDir;

use crate::{compiler::compile_path, utils::log, Settings, VM};

static TEST_EXTENSION: &str = "vtest";

#[derive(Clap, Default, Debug, Clone)]
pub struct Test {
    #[clap(short)]
    pub path: Option<String>,
}

fn is_test_file(entry: std::result::Result<walkdir::DirEntry, walkdir::Error>) -> Option<PathBuf> {
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

#[derive(Debug, Clone, Default, PartialEq, PartialOrd)]
pub struct TestRunner {
    pub run: usize,
    pub failed: usize,
    pub succeeded: usize,
}

pub fn run_test(global_settings: &Settings, cmd_settings: &Test) -> anyhow::Result<()> {
    // Search for files in path supplied by user or the one where script is run
    let path = &cmd_settings.path.as_deref().unwrap_or(".");
    let test_files: Vec<PathBuf> = WalkDir::new(path)
        .into_iter()
        .filter_map(is_test_file)
        .collect();

    log::info(format!("Going to test {} files.", test_files.len()).as_str());
    // TODO: Use utils::log instead
    for path in test_files {
        log::info(format!("Testing {:?}...", &path).as_str());
        match compile_path(path, &global_settings) {
            Ok(program) => {
                let mut vm = VM::default();
                let result = vm.interpret(program);
            }
            Err(e) => {
                dbg!(e);
            }
        }
    }

    Ok(())
}
