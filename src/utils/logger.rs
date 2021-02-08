use std::fmt::Debug;

use colored::{ColoredString, Colorize};

// Three spaces of indentation
static INDENT: &str = "   ";

pub fn indentation(indentation_level: usize) -> String {
    std::iter::repeat(INDENT).take(indentation_level).collect()
}

// const FRAME_WIDTH: usize = 50;

// pub fn in_frame(msg: &str) -> String {
//     // Additional two for the side borders
//     let border: String = std::iter::repeat('-').take(FRAME_WIDTH + 2).collect();
//     let indentation = indentation((FRAME_WIDTH / 2) - msg.len() / 2);
//     let right_indentation = if msg.len() % 2 == 0 {
//         &indentation
//     } else {
//         // If the length of the msg is not even then we need to get rid of one space
//         indentation.
//     };
//     let centered_msg = format!("{}{}{}", indentation, msg, right_indentation);
//     format!("{}\n|{}|\n{}", border, centered_msg, border)
// }

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
