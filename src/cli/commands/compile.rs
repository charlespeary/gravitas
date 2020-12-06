use std::{fs::read_to_string, io::stdin};

use anyhow::{Context, Error, Result};
use logos::Logos;

use crate::{
    bytecode::BytecodeGenerator,
    cli::commands::Compile,
    parser::{Parser, Token},
    utils::log,
    utils::Either,
    vm::VM,
};

type Compiled = Result<(), Either<Error, Vec<Error>>>;

pub fn compile(settings: Compile) -> Compiled {
    let code = read_to_string(&settings.file_path)
        .with_context(|| "Given input file doesn't exist.")
        .map_err(Either::Left)?;

    let tokens: Vec<Token> = Token::lexer(&code).collect();
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
    let _ = vm.interpret(chunk);
    Ok(())
}
