use std::{env::args, fs::read_to_string, path::Path};

use crate::{
    error::Error,
    parser::{lexer::tokenize, parser::parse},
    runtime::runtime::execute,
};

mod error;
mod parser;
mod runtime;
mod types;

fn main() -> Result<(), Error> {
    let args = args().collect::<Vec<String>>();
    if args.len() < 2 {
        return Err(Error::Custom("Usage: erw <file>".to_string()));
    }
    let file_path = Path::new(&args[1]);
    let code = read_to_string(file_path)?;

    let tokens = tokenize(&code)?;
    let ast = parse(tokens)?;
    execute()?;

    Ok(())
}
