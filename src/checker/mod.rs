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
        registry_ids::RegistryId,
        symbols::{Container, Entry, ModuleTable, Symbol},
        types::{FloatSize, IntSize, Sign, Type},
    },
};

pub(crate) struct Checker<'a> {
    pub(super) std_path: &'a Path,
    pub(super) prelude_module: &'a Path,
    pub(super) main_module: &'a Path,

    pub(super) modules: &'a HashMap<PathBuf, Vec<Statement<'a>>>,
    pub(super) tables: HashMap<&'a Path, ModuleTable<'a>>,
    pub(super) current_module: &'a Path,
    pub(super) current_scopes: Vec<HashMap<&'a str, Symbol<'a>>>,
    pub(super) returns: Vec<Type<'a>>,
    pub(super) type_aliases: HashMap<&'a str, Type<'a>>,
}

impl<'a> Checker<'a> {
    pub(crate) fn new(
        std_path: &'a Path,
        prelude_module: &'a Path,
        main_module: &'a Path,
        modules: &'a HashMap<PathBuf, Vec<Statement<'a>>>,
    ) -> Self {
        Self {
            std_path,
            prelude_module,
            main_module,

            modules,
            tables: HashMap::new(),
            current_module: main_module,
            current_scopes: Vec::new(),
            returns: Vec::new(),
            type_aliases: HashMap::new(),
        }
    }

    pub(crate) fn check(&mut self) -> Result<HashMap<&'a Path, ModuleTable<'a>>, Error> {
        self.check_module(self.current_module)?;

        Ok(take(&mut self.tables))
    }

    pub(super) fn check_module(&mut self, path: &'a Path) -> Result<(), Error> {
        if let Some(table) = self.tables.get(path)
            && !table.symbols.is_empty()
        {
            return Ok(());
        }

        let stmts = self.modules.get(path).ok_or_else(|| {
            Error::Custom(format!("Module {} not found in registry", path.display()))
        })?;

        let registry = if path == self.prelude_module {
            self.fill_primitives()
        } else {
            HashMap::new()
        };

        let symbols = if path == self.prelude_module {
            HashMap::from([(
                "prelude",
                Symbol {
                    ty: Type::Module(self.prelude_module),
                    visibility: &Visibility::Priv,
                    is_static_member: true,
                    is_mutable: false,
                },
            )])
        } else {
            HashMap::new()
        };

        self.tables.insert(
            path,
            ModuleTable {
                registry,
                symbols,
                containers: HashMap::new(),
            },
        );

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
        self.current_scopes.push(HashMap::new());
    }

    pub(super) fn exit_local_scope(
        &mut self,
        line: usize,
        col: usize,
    ) -> Result<HashMap<&'a str, Symbol<'a>>, Error> {
        if self.current_scopes.len() == 1 {
            return self.loc_error(line, col, "Cannot exit out of global scope".to_string());
        }

        Ok(self.current_scopes.pop().unwrap())
    }

    pub(super) fn exit_global_scope(&mut self) -> Result<HashMap<&'a str, Symbol<'a>>, Error> {
        if self.current_scopes.len() != 1 {
            return Err(Error::Custom(
                "Cannot exit this scope as it is not global".to_string(),
            ));
        }

        Ok(self.current_scopes.pop().unwrap())
    }

    pub(super) fn define_symbol(
        &mut self,
        id: &'a str,
        symbol: Symbol<'a>,
        line: usize,
        col: usize,
    ) -> Result<(), Error> {
        let module_table = if let Some(m) = self.tables.get(self.current_module) {
            m
        } else {
            return self.loc_error(
                line,
                col,
                format!(
                    "Module {} not found in registry",
                    self.current_module.display()
                ),
            );
        };

        if module_table
            .registry
            .contains_key(&RegistryId::from_str(id))
            || module_table.containers.contains_key(id)
        {
            return self.loc_error(
                line,
                col,
                format!("'{}' is already defined as a type or container", id),
            );
        }

        if let Some(scope) = self.current_scopes.last_mut() {
            if scope.contains_key(id) {
                return self.loc_error(
                    line,
                    col,
                    format!("Symbol {} is already defined in this scope", id),
                );
            }

            scope.insert(id, symbol);
            Ok(())
        } else {
            self.loc_error(line, col, "No active scope to define symbol in".to_string())
        }
    }

    pub(super) fn define_entry(
        &mut self,
        path: &Path,
        id: &'a str,
        entry: Entry<'a>,
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

        let registry_id = RegistryId::from_str(id);

        if module_table.registry.contains_key(&registry_id)
            || module_table.containers.contains_key(id)
            || module_table.symbols.contains_key(id)
        {
            return self.loc_error(
                line,
                col,
                format!(
                    "Entry '{}' is already defined in module {}",
                    id,
                    path.display()
                ),
            );
        }

        module_table.registry.insert(registry_id, entry);
        Ok(())
    }

    pub(super) fn define_container(
        &mut self,
        path: &Path,
        id: &'a str,
        container: Container<'a>,
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

        if module_table.containers.contains_key(id)
            || module_table
                .registry
                .contains_key(&RegistryId::from_str(id))
            || module_table.symbols.contains_key(id)
        {
            return self.loc_error(
                line,
                col,
                format!(
                    "Entry '{}' is already defined in module {}",
                    id,
                    path.display()
                ),
            );
        }

        module_table.containers.insert(id, container);
        Ok(())
    }

    pub(super) fn resolve_symbol(
        &self,
        id: &str,
        path: &Path,
        line: usize,
        col: usize,
    ) -> Result<&Symbol<'a>, Error> {
        if path == self.current_module {
            for scope in self.current_scopes.iter().rev() {
                if let Some(symbol) = scope.get(id) {
                    return Ok(symbol);
                }
            }
        } else {
            let Some(table) = self.tables.get(path) else {
                return self.loc_error(
                    line,
                    col,
                    format!("Module {} not found in registry", path.display()),
                );
            };

            if let Some(symbol) = table.symbols.get(id) {
                if matches!(symbol.visibility, Visibility::Priv) {
                    return self.loc_error(line, col, format!("Symbol '{}' is private", id));
                }
                return Ok(symbol);
            }
        }

        self.loc_error(
            line,
            col,
            format!("Symbol '{}' not found in module '{}'", id, path.display()),
        )
    }

    pub(super) fn resolve_symbol_mut(&mut self, id: &str) -> Option<&mut Symbol<'a>> {
        for scope in self.current_scopes.iter_mut().rev() {
            if let Some(symbol) = scope.get_mut(id) {
                return Some(symbol);
            }
        }
        None
    }

    pub(super) fn resolve_entry(
        &self,
        id: &'a RegistryId<'a>,
        line: usize,
        col: usize,
    ) -> Result<&Entry<'a>, Error> {
        let path = if let RegistryId::Custom {
            module: Some(module),
            ..
        } = id
        {
            if let Type::Module(p) = self
                .resolve_symbol(module, self.current_module, line, col)?
                .ty
            {
                p
            } else {
                return self.loc_error(line, col, "Expected module symbol".to_string());
            }
        } else {
            self.current_module
        };

        let Some(table) = self.tables.get(path) else {
            return self.loc_error(
                line,
                col,
                format!("Module {} not found in registry", path.display()),
            );
        };

        let Some(entry) = table.registry.get(id) else {
            return self.loc_error(
                line,
                col,
                format!(
                    "Entry '{}' not found in module {}",
                    id.to_str(),
                    path.display()
                ),
            );
        };

        if self.current_module != path && matches!(entry.visibility, Visibility::Priv) {
            return self.loc_error(line, col, format!("Entry '{}' is private", id.to_str()));
        }

        Ok(entry)
    }

    pub(super) fn resolve_entry_mut(
        &mut self,
        id: &'a RegistryId<'a>,
        line: usize,
        col: usize,
    ) -> Result<&mut Entry<'a>, Error> {
        let table_error = self.loc_error(
            line,
            col,
            format!(
                "Module {} not found in registry",
                self.current_module.display()
            ),
        );

        let entry_error = self.loc_error(
            line,
            col,
            format!(
                "Entry '{}' not found in module {}",
                id.to_str(),
                self.current_module.display()
            ),
        );

        let Some(table) = self.tables.get_mut(self.current_module) else {
            return table_error;
        };

        let Some(entry) = table.registry.get_mut(id) else {
            return entry_error;
        };

        Ok(entry)
    }

    pub(super) fn resolve_container(
        &self,
        id: &str,
        path: &Path,
        line: usize,
        col: usize,
    ) -> Result<&Container<'a>, Error> {
        let module_table = if let Some(m) = self.tables.get(path) {
            m
        } else {
            return self.loc_error(
                line,
                col,
                format!("Module {} not found in registry", path.display()),
            );
        };

        let container = if let Some(e) = module_table.containers.get(id) {
            e
        } else {
            return self.loc_error(
                line,
                col,
                format!("Container '{}' not found in module {}", id, path.display()),
            );
        };

        if self.current_module != path && matches!(container.visibility, Visibility::Priv) {
            return self.loc_error(line, col, format!("Container '{}' is private", id));
        }

        Ok(container)
    }

    pub(super) fn resolve_container_mut(
        &mut self,
        id: &str,
        line: usize,
        col: usize,
    ) -> Result<&mut Container<'a>, Error> {
        let table_error = self.loc_error(
            line,
            col,
            format!(
                "Module {} not found in registry",
                self.current_module.display()
            ),
        );

        let container_error = self.loc_error(
            line,
            col,
            format!(
                "Container '{}' not found in module {}",
                id,
                self.current_module.display()
            ),
        );

        let Some(table) = self.tables.get_mut(self.current_module) else {
            return table_error;
        };

        let Some(container) = table.containers.get_mut(id) else {
            return container_error;
        };

        Ok(container)
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

        if matches!(inferred, Type::Unknown) {
            return true;
        }

        match (explicit, inferred) {
            (
                Type::Integer {
                    size: a_size,
                    sign: a_sign,
                },
                Type::Integer {
                    size: b_size,
                    sign: b_sign,
                },
            ) => {
                if a_sign == b_sign {
                    a_size >= b_size
                } else {
                    a_size > b_size && matches!(a_sign, Sign::Signed)
                }
            }
            (
                Type::IntRange {
                    size: a_size,
                    sign: a_sign,
                },
                Type::IntRange {
                    size: b_size,
                    sign: b_sign,
                },
            ) => {
                if a_sign == b_sign {
                    a_size >= b_size
                } else {
                    a_size > b_size && matches!(a_sign, Sign::Signed)
                }
            }
            (Type::Float { size: a_size }, Type::Float { size: b_size }) => a_size >= b_size,
            (Type::Pointer(a), Type::Pointer(b)) => a == b,
            (Type::Ref(a), Type::Ref(b)) => a == b,
            (Type::Tuple(a), Type::Tuple(b)) => {
                a.len() == b.len()
                    && a.iter()
                        .zip(b.iter())
                        .all(|(l_elem, r_elem)| self.is_assignable(l_elem, r_elem))
            }
            (Type::Array(a), Type::Array(b)) => a == b,
            (
                Type::Function {
                    params: a_params,
                    return_ty: a_ty,
                },
                Type::Function {
                    params: b_params,
                    return_ty: b_ty,
                },
            ) => {
                a_params.len() == b_params.len()
                    && self.is_assignable(a_ty, b_ty)
                    && a_params
                        .iter()
                        .zip(b_params.iter())
                        .all(|(a_elem, b_elem)| self.is_assignable(b_elem, a_elem))
            }

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

        if matches!(a, Type::Unknown) {
            return Ok(b.clone());
        } else if matches!(b, Type::Unknown) {
            return Ok(a.clone());
        } else if matches!(a, Type::Done) || matches!(b, Type::Done) {
            return Ok(Type::Done);
        } else if self.is_assignable(a, b) {
            return Ok(b.clone());
        } else if self.is_assignable(b, a) {
            return Ok(a.clone());
        }

        self.loc_error(line, col, "Types are not comparable".to_string())
    }

    pub(super) fn resolve_alias(&self, ty: &Type<'a>) -> Type<'a> {
        let Type::Custom { id, .. } = ty else {
            return ty.clone();
        };

        if let Some(real_ty) = self.type_aliases.get(id) {
            return self.resolve_alias(real_ty);
        }

        ty.clone()
    }

    fn fill_primitives(&self) -> HashMap<RegistryId<'a>, Entry<'a>> {
        let primitives = [
            (RegistryId::Bool, Type::Bool),
            (
                RegistryId::UInt8,
                Type::Integer {
                    size: IntSize::B8,
                    sign: Sign::Unsigned,
                },
            ),
            (
                RegistryId::UInt16,
                Type::Integer {
                    size: IntSize::B16,
                    sign: Sign::Unsigned,
                },
            ),
            (
                RegistryId::UInt32,
                Type::Integer {
                    size: IntSize::B32,
                    sign: Sign::Unsigned,
                },
            ),
            (
                RegistryId::UInt64,
                Type::Integer {
                    size: IntSize::B64,
                    sign: Sign::Unsigned,
                },
            ),
            (
                RegistryId::UInt128,
                Type::Integer {
                    size: IntSize::B128,
                    sign: Sign::Unsigned,
                },
            ),
            (
                RegistryId::Int8,
                Type::Integer {
                    size: IntSize::B8,
                    sign: Sign::Signed,
                },
            ),
            (
                RegistryId::Int16,
                Type::Integer {
                    size: IntSize::B16,
                    sign: Sign::Signed,
                },
            ),
            (
                RegistryId::Int32,
                Type::Integer {
                    size: IntSize::B32,
                    sign: Sign::Signed,
                },
            ),
            (
                RegistryId::Int64,
                Type::Integer {
                    size: IntSize::B64,
                    sign: Sign::Signed,
                },
            ),
            (
                RegistryId::Int128,
                Type::Integer {
                    size: IntSize::B128,
                    sign: Sign::Signed,
                },
            ),
            (
                RegistryId::URange8,
                Type::IntRange {
                    size: IntSize::B8,
                    sign: Sign::Unsigned,
                },
            ),
            (
                RegistryId::URange16,
                Type::IntRange {
                    size: IntSize::B16,
                    sign: Sign::Unsigned,
                },
            ),
            (
                RegistryId::URange32,
                Type::IntRange {
                    size: IntSize::B32,
                    sign: Sign::Unsigned,
                },
            ),
            (
                RegistryId::URange64,
                Type::IntRange {
                    size: IntSize::B64,
                    sign: Sign::Unsigned,
                },
            ),
            (
                RegistryId::URange128,
                Type::IntRange {
                    size: IntSize::B128,
                    sign: Sign::Unsigned,
                },
            ),
            (
                RegistryId::Range8,
                Type::IntRange {
                    size: IntSize::B8,
                    sign: Sign::Signed,
                },
            ),
            (
                RegistryId::Range16,
                Type::IntRange {
                    size: IntSize::B16,
                    sign: Sign::Signed,
                },
            ),
            (
                RegistryId::Range32,
                Type::IntRange {
                    size: IntSize::B32,
                    sign: Sign::Signed,
                },
            ),
            (
                RegistryId::Range64,
                Type::IntRange {
                    size: IntSize::B64,
                    sign: Sign::Signed,
                },
            ),
            (
                RegistryId::Range128,
                Type::IntRange {
                    size: IntSize::B128,
                    sign: Sign::Signed,
                },
            ),
            (
                RegistryId::Float32,
                Type::Float {
                    size: FloatSize::B32,
                },
            ),
            (
                RegistryId::Float64,
                Type::Float {
                    size: FloatSize::B64,
                },
            ),
            (RegistryId::Str, Type::String),
            (RegistryId::Ptr, Type::Pointer(Box::new(Type::Unknown))),
            (RegistryId::Ref, Type::Ref(Box::new(Type::Unknown))),
            (RegistryId::Tuple, Type::Tuple(Vec::new())),
            (RegistryId::Array, Type::Array(Box::new(Type::Unknown))),
            (
                RegistryId::Func,
                Type::Function {
                    params: Vec::new(),
                    return_ty: Box::new(Type::Unknown),
                },
            ),
            (RegistryId::Node, Type::Node(Box::new(Type::Unknown))),
        ];

        let mut registry = HashMap::new();

        for (id, formal_type) in primitives {
            registry.insert(
                id,
                Entry {
                    visibility: &Visibility::Pub,
                    symbols: HashMap::new(),
                    ty: formal_type,
                },
            );
        }

        registry
    }
}
