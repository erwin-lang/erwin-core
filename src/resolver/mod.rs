use crate::{
    Path,
    error::{Error, loc_error},
    lexer::Lexer,
    parser::Parser,
    structure::parser::ast::{Expr, ExprKind, Statement, StatementKind},
};

use std::{
    collections::{HashMap, HashSet},
    fs::read_to_string,
    mem::take,
    path::PathBuf,
};

pub(crate) struct Resolver<'a> {
    std_path: &'a Path,
    main_module: &'a Path,
    loaded_modules: HashSet<PathBuf>,
    registry: HashMap<PathBuf, Vec<Statement<'a>>>,
}

impl<'a> Resolver<'a> {
    pub(crate) fn new(std_path: &'a Path, main_module: &'a Path) -> Self {
        Self {
            std_path,
            main_module,
            loaded_modules: HashSet::new(),
            registry: HashMap::new(),
        }
    }

    pub(crate) fn resolve(&mut self) -> Result<HashMap<PathBuf, Vec<Statement<'a>>>, Error> {
        self.resolve_imports(self.main_module)?;

        Ok(take(&mut self.registry))
    }

    fn resolve_imports(&mut self, module_path: &Path) -> Result<(), Error> {
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

        for mut stmt in current_stmts {
            if let StatementKind::Import {
                path,
                resolved_path,
                ..
            } = &mut stmt.kind
            {
                let parts = self.flatten_path(&path)?;
                let mut next_path = match parts.first() {
                    Some(&"std") => self
                        .std_path
                        .parent()
                        .unwrap_or(Path::new("."))
                        .to_path_buf(),
                    _ => self
                        .main_module
                        .parent()
                        .unwrap_or(Path::new("."))
                        .to_path_buf(),
                };

                for part in &parts {
                    next_path.push(part);
                    resolved_path.push(part);
                }

                next_path.set_extension("erw");

                self.resolve_imports(&next_path)?;
            }

            self.registry.get_mut(&canonical_path).unwrap().push(stmt);
        }

        Ok(())
    }

    pub(crate) fn flatten_path(&self, expr: &Expr<'a>) -> Result<Vec<&'a str>, Error> {
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
