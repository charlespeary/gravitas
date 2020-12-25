use crate::cli::commands::test::TestRunner;

/// Struct used to inject external dependencies into the VM, such as test runner
#[derive(Default, Debug)]
pub struct Injections<'a> {
    pub test_runner: Option<&'a mut TestRunner>,
}

impl<'a> Injections<'a> {
    pub fn with_test_runner(mut self, test_runner: &'a mut TestRunner) -> Self {
        self.test_runner = Some(test_runner);
        self
    }
}
