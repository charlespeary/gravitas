use anyhow::{Context, Error, Result};
use enum_as_inner::EnumAsInner;
use logos::Logos;

use std::{fs::read_to_string, io::stdin};

use crate::{
    bytecode::{BytecodeFrom, BytecodeGenerator},
    parser::{Parser, Token},
    settings::Settings,
    utils::iter::{peek_nth, PeekNth},
    vm::VM,
};

pub mod iter;

pub mod log {
    use colored::Colorize;

    use std::fmt::Debug;

    use crate::bytecode::{Opcode, Value};

    pub fn title_error(t: &str) {
        let text = format!("========= {} =========", t);
        println!("\n{}\n", text.red().bold());
    }

    pub fn title_success(t: &str) {
        let text = format!("========= {} =========", t);
        println!("\n{}\n ", text.green().bold());
    }

    pub fn body<T: Debug>(i: &T) {
        let text = format!("{:#?}", i);
        println!("{}", text.as_str().white());
    }

    pub fn vm_title<T: Debug>(text: &str, i: &T) {
        let title = text.yellow().bold();
        let body = format!("{:?}", i).yellow();
        println!("{}: {}", title, body);
    }

    pub fn vm_subtitle<T: Debug>(text: &str, i: &T) {
        let title = format!("        {}: ", text).as_str().blue().bold();
        println!("{} {}", title, format!("{:?}", i).blue());
    }
}

#[derive(Debug, Clone, Copy, EnumAsInner)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Either<L, R> {
    pub fn is_left(&self) -> bool {
        match self {
            Either::Left(_) => true,
            Either::Right(_) => false,
        }
    }

    pub fn is_right(&self) -> bool {
        match self {
            Either::Left(_) => false,
            Either::Right(_) => true,
        }
    }
}

type Compiled = Result<(), Either<Error, Vec<Error>>>;

pub fn initialize(settings: Settings) -> Compiled {
    if let Some(path) = &settings.file_path {
        let code = read_to_string(path)
            .with_context(|| "Given input file doesn't exist.")
            .map_err(Either::Left)?;

        compile(&code, &settings)?;
    } else {
        loop {
            println!("> ");

            let mut input = String::new();
            stdin()
                .read_line(&mut input)
                .with_context(|| "Couldn't read the line")
                .map_err(Either::Left)?;
            compile(&input, &settings)?;
        }
    }
    Ok(())
}

pub fn compile(code: &str, settings: &Settings) -> Compiled {
    let tokens: Vec<Token> = Token::lexer(code).collect();
    if settings.debug {
        log::title_success("LEXED");
        log::body(&tokens);
    }
    let parser = Parser::new(tokens);
    let ast = parser.parse().map_err(Either::Right)?;
    if settings.debug {
        log::title_success("PARSED");
        log::body(&ast);
    }
    let chunk = BytecodeGenerator::compile(&ast).map_err(Either::Left)?;
    if settings.debug {
        log::title_success("OPCODES");
        log::body(&chunk.code);
        log::title_success("CONSTANTS");
        log::body(&chunk.constants);
        log::title_success("INTERPRETATION");
    }
    let mut vm = VM::from(settings.clone());
    let result = vm.interpret(chunk);
    Ok(())
}

#[macro_export]
macro_rules! hashmap {
    ($($key:expr => $value:expr), *) => {{
        let mut hashmap = std::collections::HashMap::new();
        $(
          hashmap.insert($key, $value);
        )*
        hashmap
    }};
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    // macro creates correct HashMap
    #[test]
    fn hashmap() {
        let standard_map: HashMap<&str, i32> = {
            let mut map = HashMap::new();
            map.insert("one", 1);
            map.insert("two", 2);
            map.insert("three", 3);
            map
        };

        let map_from_macro = hashmap!(
            "one" => 1,
            "two" => 2,
            "three" =>3
        );

        assert_eq!(standard_map, map_from_macro);
    }
}
