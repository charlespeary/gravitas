use std::path::Path;

use anyhow::{Context, Error, Result};
use logos::Logos;

use crate::{
    bytecode::{BytecodeGenerator, Chunk, Value},
    parser::{Parser, Token},
    utils::{logger::LOGGER, Either},
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
    LOGGER.log_pdbg("TOKENS", &tokens);
    let parser = Parser::new(tokens);
    let ast = parser.parse().map_err(Either::Right)?;
    LOGGER.log_pdbg("SYNTAX TREE", &ast);
    let chunk = BytecodeGenerator::compile(&ast).map_err(Either::Left)?;
    LOGGER.log_pdbg("GENERATED CHUNK", &chunk);
    Ok(chunk)
}
