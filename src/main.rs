use std::{env::args, fs::write, path::Path};

use crate::{checker::Checker, error::Error, resolver::Resolver};

mod checker;
mod error;
mod lexer;
mod parser;
mod resolver;
mod structure;

fn main() -> Result<(), Error> {
    let args = args().collect::<Vec<String>>();
    if args.len() < 2 {
        return Err(Error::Custom("Usage: erwin <file>".to_string()));
    }

    let std_path = Path::new("/usr/lib/erwin/std/").canonicalize()?;
    let prelude_module = std_path.join("prelude.erw").canonicalize()?;
    let main_module = Path::new(&args[1]).canonicalize()?;

    let registry = Resolver::new(&std_path, &main_module).resolve()?;
    let symbol_table = Checker::new(&std_path, &prelude_module, &main_module, &registry).check()?;

    // TEST: let's save the AST tree and the symbol table to check them!
    write("/home/canfro/ast.txt", format!("{:#?}", registry))?;
    write(
        "/home/canfro/symbol_table.txt",
        format!("{:#?}", symbol_table),
    )?;

    Ok(())
}
