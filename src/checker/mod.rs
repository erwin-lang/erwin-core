pub(super) mod expr;
pub(super) mod statement;

use std::{
    collections::HashMap,
    mem::take,
    path::{Path, PathBuf},
};

use crate::{
    error::Error,
    structure::{
        ast::{ExprKind, Statement, Visibility},
        symbols::{ModuleTable, Scope, ScopedSymbol, StaticEntry},
        types::Type,
    },
};

pub(crate) struct Checker<'a> {
    pub(super) modules: &'a HashMap<PathBuf, Vec<Statement<'a>>>,
    pub(super) tables: HashMap<&'a Path, ModuleTable<'a>>,
    pub(super) current_module: &'a Path,
    pub(super) current_scopes: Vec<Scope<'a>>,
    pub(super) returns: Vec<Type<'a>>,
}

impl<'a> Checker<'a> {
    pub(crate) fn new(
        registry: &'a HashMap<PathBuf, Vec<Statement<'a>>>,
        main_mod: &'a Path,
    ) -> Self {
        Self {
            modules: registry,
            tables: HashMap::new(),
            current_module: main_mod,
            current_scopes: Vec::new(),
            returns: Vec::new(),
        }
    }

    pub(crate) fn check(&mut self) -> Result<HashMap<&'a Path, ModuleTable<'a>>, Error> {
        let primitives_id = vec![
            "Bool",
            "UInt8",
            "UInt16",
            "UInt32",
            "UInt64",
            "UInt128",
            "Int8",
            "Int16",
            "Int32",
            "Int64",
            "Int128",
            "URange8",
            "URange16",
            "URange32",
            "URange64",
            "URange128",
            "Range8",
            "Range16",
            "Range32",
            "Range64",
            "Range128",
            "Float32",
            "Float64",
            "Str",
            "Ptr",
            "Ref",
            "Tuple",
            "Array",
            "Func",
            "Node",
        ];

        for id in primitives_id {
            self.define_static(
                self.current_module,
                id,
                StaticEntry {
                    visibility: &Visibility::Pub,
                    members: HashMap::new(),
                },
                0,
                0,
            )?;
        }

        self.check_module(self.current_module)?;

        Ok(take(&mut self.tables))
    }

    pub(super) fn check_module(&mut self, path: &'a Path) -> Result<(), Error> {
        if self.tables.contains_key(path) {
            return Ok(());
        }

        let stmts = self.modules.get(path).ok_or_else(|| {
            Error::Custom(format!("Module {} not found in registry", path.display()))
        })?;

        let old_module = self.current_module;
        self.current_module = path;
        self.enter_scope();

        for stmt in stmts.as_slice() {
            self.check_global_statements(stmt)?;
        }

        for stmt in stmts.as_slice() {
            self.check_statement(stmt)?;
        }

        let global_symbols = self.exit_global_scope()?;

        if let Some(table) = self.tables.get_mut(path) {
            table.symbols = global_symbols;
        }

        self.current_module = old_module;

        Ok(())
    }

    pub(super) fn enter_scope(&mut self) {
        self.current_scopes.push(Scope {
            symbols: HashMap::new(),
        });
    }

    pub(super) fn exit_local_scope(&mut self, line: usize, col: usize) -> Result<Scope<'a>, Error> {
        if self.current_scopes.len() == 1 {
            return self.loc_error(line, col, "Cannot exit out of global scope".to_string());
        }

        Ok(self.current_scopes.pop().unwrap())
    }

    pub(super) fn exit_global_scope(&mut self) -> Result<Scope<'a>, Error> {
        if self.current_scopes.len() != 1 {
            return Err(Error::Custom(
                "Cannot exit this scope as it is not global".to_string(),
            ));
        }

        Ok(self.current_scopes.pop().unwrap())
    }

    pub(super) fn define(
        &mut self,
        id: &'a str,
        symbol: ScopedSymbol<'a>,
        line: usize,
        col: usize,
    ) -> Result<(), Error> {
        if let Some(scope) = self.current_scopes.last_mut() {
            if scope.symbols.contains_key(id) {
                return self.loc_error(
                    line,
                    col,
                    format!("Symbol {} is already defined in this scope", id),
                );
            }

            scope.symbols.insert(id, symbol);
            Ok(())
        } else {
            self.loc_error(line, col, "No active scope to define symbol in".to_string())
        }
    }

    pub(super) fn define_static(
        &mut self,
        path: &Path,
        id: &'a str,
        entry: StaticEntry<'a>,
        line: usize,
        col: usize,
    ) -> Result<(), Error> {
        let module_table = if let Some(m) = self.tables.get_mut(path) {
            m
        } else {
            return self.loc_error(
                line,
                col,
                format!("Module {} not found in registry", path.display()),
            );
        };

        if module_table.registry.contains_key(id) {
            return self.loc_error(
                line,
                col,
                format!(
                    "Static symbol '{}' is already defined in module {}",
                    id,
                    path.display()
                ),
            );
        }

        module_table.registry.insert(id, entry);
        Ok(())
    }

    pub(super) fn resolve(&self, id: &str) -> Option<&ScopedSymbol<'a>> {
        for scope in self.current_scopes.iter().rev() {
            if let Some(symbol) = scope.symbols.get(id) {
                return Some(symbol);
            }
        }
        None
    }

    pub(super) fn resolve_mut(&mut self, id: &str) -> Option<&mut ScopedSymbol<'a>> {
        for scope in self.current_scopes.iter_mut().rev() {
            if let Some(symbol) = scope.symbols.get_mut(id) {
                return Some(symbol);
            }
        }
        None
    }

    pub(super) fn resolve_external(
        &self,
        id: &str,
        path: &Path,
        line: usize,
        col: usize,
    ) -> Result<&ScopedSymbol<'a>, Error> {
        let module_table = if let Some(m) = self.tables.get(path) {
            m
        } else {
            return self.loc_error(
                line,
                col,
                format!("Module {} not found in registry", path.display()),
            );
        };

        let symbol = if let Some(s) = module_table.symbols.symbols.get(id) {
            s
        } else {
            return self.loc_error(
                line,
                col,
                format!("Symbol '{}' not found in module {}", id, path.display()),
            );
        };

        if matches!(symbol.visibility, Visibility::Priv) {
            return self.loc_error(line, col, format!("Symbol '{}' is private", id));
        }

        Ok(symbol)
    }

    pub(super) fn resolve_static(
        &self,
        id: &str,
        path: &Path,
        line: usize,
        col: usize,
    ) -> Result<&StaticEntry<'a>, Error> {
        let table = if let Some(m) = self.tables.get(path) {
            m
        } else {
            return self.loc_error(
                line,
                col,
                format!("Module {} not found in registry", path.display()),
            );
        };

        let entry = if let Some(e) = table.registry.get(id) {
            e
        } else {
            return self.loc_error(
                line,
                col,
                format!("Entry '{}' not found in module {}", id, path.display()),
            );
        };

        if self.current_module != path && matches!(entry.visibility, Visibility::Priv) {
            return self.loc_error(line, col, format!("Entry '{}' is private", id));
        }

        Ok(entry)
    }

    pub(super) fn resolve_static_mut(
        &mut self,
        id: &str,
        path: &Path,
        line: usize,
        col: usize,
    ) -> Result<&mut StaticEntry<'a>, Error> {
        let table = if let Some(m) = self.tables.get(path) {
            m
        } else {
            return self.loc_error(
                line,
                col,
                format!("Module {} not found in registry", path.display()),
            );
        };

        let entry = if let Some(e) = table.registry.get(id) {
            e
        } else {
            return self.loc_error(
                line,
                col,
                format!("Entry '{}' not found in module {}", id, path.display()),
            );
        };

        if self.current_module != path && matches!(entry.visibility, Visibility::Priv) {
            return self.loc_error(line, col, format!("Entry '{}' is private", id));
        }

        let entry_mut = self
            .tables
            .get_mut(path)
            .unwrap()
            .registry
            .get_mut(id)
            .unwrap();

        Ok(entry_mut)
    }

    pub(super) fn loc_error<T>(&self, line: usize, col: usize, msg: String) -> Result<T, Error> {
        Err(Error::Custom(format!("[{}, {}] {}", line, col, msg)))
    }

    pub(super) fn is_literal(&self, expr: &ExprKind<'a>) -> bool {
        match expr {
            ExprKind::Number(_) | ExprKind::String(_) | ExprKind::Bool(_) => true,
            ExprKind::Tuple(elems) => elems.iter().all(|elem| self.is_literal(&elem.kind)),
            _ => false,
        }
    }

    pub(super) fn is_assignable(&self, explicit: &Type<'a>, inferred: &Type<'a>) -> bool {
        if explicit == inferred {
            return true;
        }

        if matches!(explicit, Type::Universal) {
            return true;
        }

        if matches!(inferred, Type::Null) {
            return true;
        }

        match (explicit, inferred) {
            (
                Type::Integer {
                    size: t_size,
                    sign: t_sign,
                },
                Type::Integer {
                    size: s_size,
                    sign: s_sign,
                },
            ) => t_size >= s_size && t_sign >= s_sign,
            (
                Type::IntRange {
                    size: t_size,
                    sign: t_sign,
                },
                Type::IntRange {
                    size: s_size,
                    sign: s_sign,
                },
            ) => t_size >= s_size && t_sign >= s_sign,
            (Type::Float { size: t_size }, Type::Float { size: s_size }) => t_size >= s_size,
            _ => false,
        }
    }

    pub(super) fn join_ty(
        &self,
        a: &Type<'a>,
        b: &Type<'a>,
        line: usize,
        col: usize,
    ) -> Result<Type<'a>, Error> {
        if a == b {
            return Ok(a.clone());
        }

        if matches!(a, Type::Null) {
            return Ok(b.clone());
        } else if matches!(b, Type::Null) {
            return Ok(a.clone());
        } else if matches!(a, Type::Universal) || matches!(b, Type::Universal) {
            return Ok(Type::Universal);
        } else if self.is_assignable(a, b) {
            return Ok(b.clone());
        } else if self.is_assignable(b, a) {
            return Ok(a.clone());
        }

        self.loc_error(line, col, "Types are not comparable".to_string())
    }
}
