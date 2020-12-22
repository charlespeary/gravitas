use crate::cli::commands::test::TestRunner;

/// Struct used to inject external dependencies into the VM, such as test runner
#[derive(Default, Debug)]
pub struct Injections<'a> {
    pub test_runner: Option<&'a mut TestRunner>,
}

impl<'a> Injections<'a> {
    pub fn get_test_runner(&mut self) -> &mut TestRunner {
        self.test_runner
            .as_deref_mut()
            .expect("Tried to access test runner in an environment where it's not injected")
    }
}

impl<'a> From<&'a mut TestRunner> for Injections<'a> {
    fn from(test_runner: &'a mut TestRunner) -> Self {
        Self {
            test_runner: Some(test_runner),
        }
    }
}
