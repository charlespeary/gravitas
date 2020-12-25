use std::path::PathBuf;

use clap::Clap;
use walkdir::WalkDir;

use crate::{compiler::compile_path, Settings, utils::log, VM, vm::injections::Injections};

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
    let vm_injections = Injections::from(&mut test_runner);
    let mut vm = VM::default().with_injections(vm_injections).with_settings(global_settings.clone());

    log::title(format!("Going to test {} files:", test_files.len()).as_str());
    // Run code in all of the matched files
    // Fold them into count of invalid and valid programs
    let (invalid_programs, valid_programs) = test_files.iter().fold((0, 0), |acc, path| {
        match compile_path(&path, &global_settings) {
            Ok(program) => {
                log::subtitle(format!("{:?}", &path).as_str());
                let _ = vm.interpret(program);
                (acc.0, acc.1 + 1)
            }
            Err(_) => {
                log::error(format!("{:?} contains invalid code.", &path).as_str());
                (acc.0 + 1, acc.1)
            }
        }
    });

    // Display a summary of the tests we just ran

    log::title("Summary:");
    log::title_indent("Programs:", 1);
    log::info_indent(
        format!("Total: {}", valid_programs + invalid_programs).as_str(),
        2,
    );
    log::success_indent(format!("Valid: {}", valid_programs).as_str(), 2);
    log::error_indent(format!("Invalid: {}", invalid_programs).as_str(), 2);
    log::title_indent("Assertions:", 1);
    log::info_indent(format!("Ran: {}", test_runner.ran).as_str(), 2);
    log::success_indent(format!("Succeeded: {}", test_runner.succeeded).as_str(), 2);
    log::error_indent(format!("Failed: {}", test_runner.failed).as_str(), 2);

    // If there are any invalid tests or programs then just exit the process with error
    // If not then exit successfully
    if invalid_programs > 0 || test_runner.failed > 0 {
        std::process::exit(1);
    } else {
        std::process::exit(0);
    }
}
