use std::fs::read_to_string;
use std::io::stdin;

use anyhow::{Context, Error, Result};
use logos::Logos;

use crate::bytecode::BytecodeGenerator;
use crate::parser::{Parser, Token};
use crate::settings::Settings;
use crate::vm::VM;

pub use self::iter::{peek_nth, PeekNth};

mod iter;

#[derive(Debug)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

pub fn initialize(settings: &Settings) -> Result<()> {
    if let Some(path) = &settings.file_path {
        let code = read_to_string(path).with_context(|| "Given input file doesn't exist.")?;
        let result = compile(&code);
        println!("compiled: {:#?}", result);
    } else {
        loop {
            println!("> ");

            let mut input = String::new();
            match stdin().read_line(&mut input) {
                Ok(n) => {
                    let result = compile(&input);
                    println!("compiled: {:#?}", result);
                }
                Err(error) => {
                    println!("error: {}", error);
                }
            }
        }
    }
    Ok(())
}

pub fn compile(code: &str) -> Result<(), Either<Error, Vec<Error>>> {
    let tokens: Vec<Token> = Token::lexer(code).collect();
    let parser = Parser::new(tokens);
    let ast = parser.parse().map_err(Either::Right)?;
    println!("Parsed: {:#?}", ast);
    let mut bg = BytecodeGenerator::new();
    let chunk = bg.generate(&ast).map_err(Either::Left)?;
    println!("GENERATED: {:#?}", chunk);
    let mut vm = VM::default();
    let interpreted = vm.interpret(&chunk);
    println!("INTERPRETED: {:#?}", interpreted);
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
