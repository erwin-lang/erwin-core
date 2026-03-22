use std::{env::args, fs::read_to_string, path::Path};

use crate::{error::Error, lexer::Lexer, parser::Parser, runtime::runtime::execute};

mod checker;
mod error;
mod lexer;
mod parser;
mod runtime;
mod structure;

fn main() -> Result<(), Error> {
    let args = args().collect::<Vec<String>>();
    if args.len() < 2 {
        return Err(Error::Custom("Usage: erw <file>".to_string()));
    }
    let file_path = Path::new(&args[1]);
    let code = read_to_string(file_path)?;

    let mut lexer = Lexer::new(code.as_str());
    let tokens = lexer.tokenize()?;

    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;
    execute()?;

    Ok(())
}
