use std::{
    collections::{HashMap, HashSet},
    env::args,
    fs::read_to_string,
    path::{Path, PathBuf},
};

use crate::{
    checker::Checker,
    error::Error,
    lexer::Lexer,
    parser::Parser,
    structure::ast::{Statement, StatementKind},
};

mod checker;
mod error;
mod lexer;
mod parser;
mod structure;

fn resolve_imports<'a>(
    module_path: &Path,
    loaded_modules: &mut HashSet<PathBuf>,
    registry: &mut HashMap<PathBuf, Vec<Statement<'a>>>,
) -> Result<(), Error> {
    let canonical_path = module_path.canonicalize()?;

    if loaded_modules.contains(&canonical_path) {
        return Ok(());
    }
    loaded_modules.insert(canonical_path.clone());
    registry.insert(canonical_path.clone(), Vec::new());

    let code = read_to_string(&canonical_path)?;
    let leaked_code = Box::leak(code.into_boxed_str());
    let tokens = Lexer::new(leaked_code).tokenize()?;
    let current_stmts = Parser::new(tokens).parse()?;

    for stmt in current_stmts {
        if let StatementKind::Import { alias: _, path } = stmt.kind {
            let mut next_path = canonical_path
                .parent()
                .unwrap_or(Path::new("."))
                .to_path_buf();

            for part in path {
                next_path.push(part);
            }

            next_path.set_extension("erw");
            resolve_imports(&next_path, loaded_modules, registry)?;
        } else {
            registry.get_mut(&canonical_path).unwrap().push(stmt);
        }
    }

    Ok(())
}

fn main() -> Result<(), Error> {
    let args = args().collect::<Vec<String>>();
    if args.len() < 2 {
        return Err(Error::Custom("Usage: erw <file>".to_string()));
    }

    let main_path = Path::new(&args[1]);
    let mut loaded_modules = HashSet::new();
    let mut registry = HashMap::new();

    resolve_imports(main_path, &mut loaded_modules, &mut registry)?;

    let symbol_table = Checker::new(&registry, main_path).check()?;
    // Then Assembler::new(...) here, when it's implemented

    Ok(())
}
