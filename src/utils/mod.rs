use std::fs::read_to_string;
use std::io::stdin;

use anyhow::{Context, Result};
use logos::Logos;

use crate::bytecode::BytecodeGenerator;
use crate::parser::{Parser, Token, Visitor};
use crate::settings::Settings;
use crate::vm::VM;

pub use self::iter::{peek_nth, PeekNth};

mod iter;

pub fn initialize(settings: &Settings) -> Result<()> {
    if let Some(path) = &settings.file_path {
        let code = read_to_string(path).with_context(|| "Given input file doesn't exist.")?;
        compile(&code);
    } else {
        loop {
            println!("> ");

            let mut input = String::new();
            match stdin().read_line(&mut input) {
                Ok(n) => compile(&input),
                Err(error) => println!("error: {}", error),
            }
        }
    }
    Ok(())
}

pub fn compile(code: &str) {
    let tokens: Vec<Token> = Token::lexer(code).collect();
    let mut parser = Parser::new(tokens);
    let expr = parser.compile().unwrap();
    println!("PARSED: {:#?}", expr);
    let chunk = BytecodeGenerator::default()
        .visit(&expr)
        .expect("I need a chunk!");
    println!("GENERATED: {:#?}", chunk);
    let mut vm = VM::new();
    let result = vm.interpret(&chunk);
    println!("INTERPRETED: {:#?}", result);
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
