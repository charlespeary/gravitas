use std::fmt::Debug;

use colored::{ColoredString, Colorize};

// Three spaces of indentation
static INDENT: &str = "   ";

pub fn indentation(indentation_level: usize) -> String {
    std::iter::repeat(INDENT).take(indentation_level).collect()
}

pub struct Log {
    msg: ColoredString,
}

impl Log {
    pub fn title(mut self) -> Self {
        self.msg = self.msg.bold();
        self
    }

    pub fn success(mut self) -> Self {
        self.msg = self.msg.green();
        self
    }

    pub fn info(mut self) -> Self {
        self.msg = self.msg.yellow();
        self
    }

    pub fn error(mut self) -> Self {
        self.msg = self.msg.red();
        self
    }

    pub fn log(self) {
        println!("{}", self.msg);
    }

    pub fn indent(self, depth: usize) {
        println!("{}{}", indentation(depth), self.msg);
    }
}

pub fn log<M: Into<String>>(msg: M) -> Log {
    Log {
        msg: msg.into().as_str().normal(),
    }
}

pub fn dbg<I: Debug>(item: I) -> Log {
    Log {
        msg: format!("{:#?}", item).as_str().normal(),
    }
}
