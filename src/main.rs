use std::{env::args, path::Path};

use crate::{arena::Arena, checker::Checker, error::Error, resolver::Resolver};

mod arena;
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
    let arena = Box::leak(Box::new(Arena::new()));

    let registry = Resolver::new(&arena, &std_path, &main_module).resolve()?;

    let mut checker = Checker::new(&arena, &std_path, &prelude_module, &main_module, &registry);
    let (instructions, literals) = checker.check()?;
    checker.debug()?;

    Ok(())
}
