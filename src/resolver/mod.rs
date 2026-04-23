use crate::{
    Path,
    error::{Error, loc_error},
    lexer::Lexer,
    parser::Parser,
    structure::ast::{Expr, ExprKind, Statement, StatementKind},
};

use std::{
    collections::{HashMap, HashSet},
    fs::read_to_string,
    path::PathBuf,
};

pub(crate) struct Resolver<'a> {
    std_path: PathBuf,
    main_path: PathBuf,
    loaded_modules: HashSet<PathBuf>,
    registry: HashMap<PathBuf, Vec<Statement<'a>>>,
}

impl<'a> Resolver<'a> {
    pub(crate) fn new(std_path: PathBuf, main_path: PathBuf) -> Self {
        Self {
            std_path,
            main_path,
            loaded_modules: HashSet::new(),
            registry: HashMap::new(),
        }
    }

    pub(crate) fn resolve(&mut self, module_path: PathBuf) -> Result<(), Error> {
        let canonical_path = module_path.canonicalize()?;

        if self.loaded_modules.contains(&canonical_path) {
            return Ok(());
        }

        self.loaded_modules.insert(canonical_path.clone());
        self.registry.insert(canonical_path.clone(), Vec::new());

        let code = read_to_string(&canonical_path)?;
        let leaked_code = Box::leak(code.into_boxed_str());
        let tokens = Lexer::new(leaked_code).tokenize()?;
        let current_stmts = Parser::new(tokens).parse()?;

        for stmt in current_stmts {
            if let StatementKind::Import { path, .. } = &stmt.kind {
                let flattened_path = self.flatten_path(path)?;
                let mut next_path = match flattened_path.first() {
                    Some(&"std") => self
                        .std_path
                        .parent()
                        .unwrap_or(Path::new("."))
                        .to_path_buf(),
                    Some(_) => self
                        .main_path
                        .parent()
                        .unwrap_or(Path::new("."))
                        .to_path_buf(),
                    None => unreachable!(),
                };

                for part in flattened_path {
                    next_path.push(part);
                }

                next_path.set_extension("erw");
                self.resolve(next_path)?;
            }

            self.registry.get_mut(&canonical_path).unwrap().push(stmt);
        }

        Ok(())
    }

    pub(crate) fn flatten_path(&self, expr: &'a Expr<'a>) -> Result<Vec<&'a str>, Error> {
        match &expr.kind {
            ExprKind::Identifier(id) => Ok(vec![id]),
            ExprKind::StaticAccess { target, member } => {
                let mut path = self.flatten_path(target)?;
                path.push(member);
                Ok(path)
            }
            _ => loc_error(
                expr.line,
                expr.col,
                "Invalid import path: expected identifier or static access expression",
            ),
        }
    }
}
