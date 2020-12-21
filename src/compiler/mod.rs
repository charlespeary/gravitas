use std::path::Path;

use anyhow::{Context, Error, Result};
use logos::Logos;

use crate::{
    bytecode::BytecodeGenerator,
    parser::{Parser, Token},
    Settings,
    utils::Either,
    utils::log, VM,
};
use crate::bytecode::{Chunk, Value};

pub type ProgramOutput = Result<Value>;

type Program = Chunk;

type Compiled = Result<Program, Either<Error, Vec<Error>>>;

pub fn compile_path<P: AsRef<Path>>(path: P, settings: &Settings) -> Compiled {
    let code = std::fs::read_to_string(path)
        .with_context(|| "Given input file doesn't exist.")
        .map_err(Either::Left)?;
    compile(&code, settings)
}

pub fn compile(code: &str, settings: &Settings) -> Compiled {
    let tokens: Vec<Token> = Token::lexer(&code).collect();
    if settings.debug {
        log::success("LEXED");
        log::body(&tokens);
    }
    let parser = Parser::new(tokens);
    let ast = parser.parse().map_err(Either::Right)?;
    if settings.debug {
        log::success("PARSED");
        log::body(&ast);
    }
    let chunk = BytecodeGenerator::compile(&ast).map_err(Either::Left)?;
    if settings.debug {
        log::success("OPCODES");
        log::body(&chunk.code);
        log::success("CONSTANTS");
        log::body(&chunk.constants);
        log::success("INTERPRETATION");
    }

    Ok(chunk.into())
}
