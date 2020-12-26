use std::path::PathBuf;

use clap::Clap;
use walkdir::WalkDir;

use crate::{compiler::compile_path, Settings, utils::logger::log, VM, vm::utilities::Utilities};

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

#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct TestRunner {
    pub ran: usize,
    pub failed: usize,
    pub succeeded: usize,
}

impl TestRunner {
    pub fn success(&mut self) {
        self.succeeded += 1;
    }
    pub fn run(&mut self) {
        self.ran += 1;
    }
    pub fn failure(&mut self) {
        self.failed += 1;
    }
}

pub fn run_test(global_settings: &Settings, cmd_settings: &Test) {
    // Search for files in path supplied by user or the one where script is run
    let path = &cmd_settings.path.as_deref().unwrap_or(".");
    // Get all paths to files containing .vtest extension
    let test_files: Vec<PathBuf> = WalkDir::new(path)
        .into_iter()
        .filter_map(is_test_file)
        .collect();
    let mut test_runner = TestRunner::default();

    log(&format!("Going to test {} files:", test_files.len())).title().log();
    // Run code in all of the matched files
    // Fold them into count of invalid and valid programs
    let (invalid_programs, valid_programs) = {
        let mut utilities = Utilities::default().with_settings(&global_settings).with_test_runner(&mut test_runner);
        let mut vm = VM::default().with_utilities(&mut utilities);

        test_files.iter().fold((0, 0), |acc, path| {
            match compile_path(&path, &global_settings) {
                Ok(program) => {
                    log(&format!("{:?}", &path)).info().indent(1);
                    let _ = vm.interpret(program);
                    (acc.0, acc.1 + 1)
                }
                Err(_) => {
                    log(&format!("{:?} contains invalid code.", &path)).error().indent(1);
                    (acc.0 + 1, acc.1)
                }
            }
        })
    };

    // Display a summary of the tests we just ran
    log("Summary:").title().log();
    log("Programs:").indent(1);
    log(&format!("Total: {}", valid_programs + invalid_programs)).info().indent(2);
    log(&format!("Valid: {}", valid_programs)).success().indent(2);
    log(&format!("Invalid: {}", invalid_programs)).error().indent(2);
    log("Assertions:").indent(1);
    log(&format!("Ran: {}", test_runner.ran)).info().indent(2);
    log(&format!("Succeeded: {}", test_runner.succeeded)).success().indent(2);
    log(&format!("Failed: {}", test_runner.failed)).error().indent(2);

    // If there are any invalid tests or programs then just exit the process with error
    // If not then exit successfully
    if invalid_programs > 0 || test_runner.failed > 0 {
        std::process::exit(1);
    } else {
        std::process::exit(0);
    }
}
