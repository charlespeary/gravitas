use std::fmt::{Debug, Display};

use clap::Clap;
use colored::{ColoredString, Colorize};
use lazy_static::lazy_static;

use crate::Settings;

// Three spaces of indentation
static INDENT: &str = "   ";

pub fn indentation(indentation_level: usize) -> String {
    std::iter::repeat(INDENT).take(indentation_level).collect()
}

const FRAME_WIDTH: usize = 50;

pub fn in_frame(msg: &str) -> String {
    // Additional two for the side borders
    let border: String = std::iter::repeat('-').take(FRAME_WIDTH + 2).collect();
    let indentation = indentation((FRAME_WIDTH / 2) - msg.len() / 2);
    // let right_indentation = if msg.len() % 2 == 0 {
    //     &indentation
    // } else {
    // If the length of the msg is not even then we need to get rid of one space
    // indentation.
    // };
    let centered_msg = format!("{}{}{}", indentation, msg, indentation);
    format!("{}\n|{}|\n{}", border, centered_msg, border)
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

impl ToString for Log {
    fn to_string(&self) -> String {
        self.msg.to_string()
    }
}

fn make_chars(c: char, amount: usize) -> String {
    std::iter::repeat(c).take(amount).collect()
}

const TITLE_CELL_WIDTH: usize = 25;

pub struct Logger {
    pub debug: bool,
}

impl Logger {
    pub fn log(&self, title: &str, msg: &str) {
        if self.debug {
            let terminal_width = term_size::dimensions().map(|d| d.0).unwrap_or(0);
            let title_padding: String = make_chars(' ', TITLE_CELL_WIDTH - title.len());
            let title = format!("{}{}|", title, title_padding).as_str().cyan();

            let border: String = std::iter::repeat('-').take(terminal_width).collect();

            println!("{} {}", title, msg);
            println!("{}", border);
        }
    }

    pub fn log_title(&self, title: &str) {
        self.log(title, "###");
    }

    pub fn log_dsp<M: Display>(&self, title: &str, msg: M) {
        self.log(title, &format!("{}", msg));
    }

    pub fn log_dbg<M: Debug>(&self, title: &str, msg: M) {
        self.log(title, &format!("{:?}", msg));
    }

    pub fn log_pdbg<M: Debug>(&self, title: &str, msg: M) {
        self.log(title, &format!("{:#?}", msg));
    }
}

lazy_static! {
    pub static ref LOGGER: Logger = {
        let settings = Settings::parse();
        Logger {
            debug: settings.debug,
        }
    };
}
