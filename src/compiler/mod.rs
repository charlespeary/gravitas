use std::path::Path;

use anyhow::{Context, Error, Result};
use logos::Logos;

use crate::utils::logger::log;
use crate::{
    bytecode::{BytecodeGenerator, Chunk, Value},
    parser::{Parser, Token},
    utils::{logger, Either},
    Settings, VM,
};

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
        logger::log("TOKENS:").title().log();
        logger::dbg(&tokens).indent(1);
    }
    let parser = Parser::new(tokens);
    let ast = parser.parse().map_err(Either::Right)?;
    if settings.debug {
        logger::log("SYNTAX TREE:").title().log();
        logger::dbg(&ast).indent(1);
    }
    let chunk = BytecodeGenerator::compile(&ast).map_err(Either::Left)?;
    if settings.debug {
        logger::log("BYTECODE:").title().log();
        logger::dbg(&chunk).indent(1);
    }
    Ok(chunk.into())
}
