use crate::{cli::commands::test::TestRunner, Settings};

/// Struct used to inject external dependencies into the VM, such as test runner
#[derive(Debug, Default)]
pub struct Utilities<'a> {
    pub test_runner: Option<&'a mut TestRunner>,
    pub settings: Option<&'a Settings>,
}

impl<'a> Utilities<'a> {
    pub fn with_settings(mut self, settings: &'a Settings) -> Self {
        self.settings = Some(settings);
        self
    }

    pub fn with_test_runner(mut self, test_runner: &'a mut TestRunner) -> Self {
        self.test_runner = Some(test_runner);
        self
    }
}

