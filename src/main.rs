use std::{
    collections::{HashMap, HashSet},
    env::args,
    fs::{read_to_string, write},
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
    std_path: &Path,
    main_path: &Path,
    module_path: &Path,
    loaded_modules: &mut HashSet<PathBuf>,
    registry: &mut HashMap<PathBuf, Vec<Statement<'a>>>,
) -> Result<(), Error> {
    let module_path = module_path.canonicalize()?;

    if loaded_modules.contains(&module_path) {
        return Ok(());
    }

    loaded_modules.insert(module_path.clone());
    registry.insert(module_path.clone(), Vec::new());

    let code = read_to_string(&module_path)?;
    let leaked_code = Box::leak(code.into_boxed_str());
    let tokens = Lexer::new(leaked_code).tokenize()?;
    let current_stmts = Parser::new(tokens).parse()?;

    for stmt in current_stmts {
        if let StatementKind::Import { alias: _, path } = stmt.kind {
            let mut next_path = match path.first() {
                Some(elem) if *elem == "std" => {
                    std_path.parent().unwrap_or(Path::new(".")).to_path_buf()
                }
                Some(_) => main_path.parent().unwrap_or(Path::new(".")).to_path_buf(),
                None => unreachable!(),
            };

            for part in path {
                next_path.push(part);
            }

            next_path.set_extension("erw");
            resolve_imports(std_path, main_path, &next_path, loaded_modules, registry)?;
        } else {
            registry.get_mut(&module_path).unwrap().push(stmt);
        }
    }

    Ok(())
}

fn main() -> Result<(), Error> {
    let args = args().collect::<Vec<String>>();
    if args.len() < 2 {
        return Err(Error::Custom("Usage: erwin <file>".to_string()));
    }

    let std_path = Path::new("/usr/lib/erwin/std/").canonicalize()?;
    let prelude_module = std_path.join("prelude.erw").canonicalize()?;
    let main_module = Path::new(&args[1]).canonicalize()?;

    let mut loaded_modules = HashSet::new();
    let mut registry = HashMap::new();

    resolve_imports(
        &std_path,
        &main_module,
        &main_module,
        &mut loaded_modules,
        &mut registry,
    )?;

    let symbol_table = Checker::new(&std_path, &prelude_module, &main_module, &registry).check()?;

    // TEST: let's save the AST tree and the symbol table to check them!
    write("test/ast.txt", format!("{:#?}", registry))?;
    write("test/symbol_table.txt", format!("{:#?}", symbol_table))?;

    Ok(())
}
