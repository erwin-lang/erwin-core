use std::{env::args, path::Path};

use crate::{arena::Arena, error::Error};

mod arena;
mod error;
mod lexer;

fn main() -> Result<(), Error> {
    let args = args().collect::<Vec<String>>();
    if args.len() < 2 {
        return Err(Error::Custom("Usage: erw <file>".to_string()));
    }

    let std_path = Path::new("/usr/lib/erwin/std/").canonicalize()?;
    let prelude_module = std_path.join("prelude.erw").canonicalize()?;
    let main_module = Path::new(&args[1]).canonicalize()?;
    let arena = Box::leak(Box::new(Arena::new()));

    Ok(())
}
