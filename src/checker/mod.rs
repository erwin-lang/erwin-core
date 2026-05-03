pub(super) mod expr;
pub(super) mod stmt_first;
pub(super) mod stmt_second;
pub(super) mod types;

use std::{
    collections::{HashMap, HashSet},
    fs::write,
    mem::take,
    path::{Path, PathBuf},
};

use crate::{
    arena::Arena,
    error::{Error, loc_error},
    structure::{
        checker::{
            instruction::Instruction,
            literal::Literal,
            module_table::{Container, ModuleTable, Scope, ScopeSymbol, Type, TypeSymbol},
        },
        parser::ast::{Statement, Visibility},
    },
};

pub(crate) struct Checker<'a> {
    // Received from Resolver, read-only
    arena: &'a Arena<'a>,
    std_path: &'a Path,
    prelude_module: &'a Path,
    main_module: &'a Path,
    registry: &'a HashMap<PathBuf, Vec<Statement<'a>>>, // Module registry, this is what we will walk over and check

    // For checking logic (internal to the checker, the assembler won't know about these)
    current_module: &'a Path,           // Module being currently checked
    checked_modules: HashSet<&'a Path>, // Guard against circular imports
    return_types: Vec<Type<'a>>,        // Return types stack for handling nested lambdas
    module_tables: HashMap<&'a Path, ModuleTable<'a>>, // Module tables holding scoped symbols, type symbols, and containers
    scope_index: usize,                                // Index of the current scope being checked

    // This is what we want to build while we check and pass to the assembler
    instructions: Vec<Instruction>, // Assembler will read these as actions holding metadata (they're basically pre-assembly)
    literals: Vec<Literal<'a>>,     // Assembler will read these as non-computed values
}

impl<'a> Checker<'a> {
    pub(crate) fn new(
        arena: &'a Arena<'a>,
        std_path: &'a Path,
        prelude_module: &'a Path,
        main_module: &'a Path,
        registry: &'a HashMap<PathBuf, Vec<Statement<'a>>>,
    ) -> Self {
        Self {
            arena,
            std_path,
            prelude_module,
            main_module,
            registry,

            current_module: main_module,
            checked_modules: HashSet::new(),
            return_types: Vec::new(),
            module_tables: HashMap::new(),
            scope_index: 0,

            instructions: Vec::new(),
            literals: Vec::new(),
        }
    }

    pub(crate) fn check(&mut self) -> Result<(Vec<Instruction>, Vec<Literal<'a>>), Error> {
        self.check_module(self.current_module, 0, 0);

        let instructions = take(&mut self.instructions);
        let literals = take(&mut self.literals);
        Ok((instructions, literals))
    }

    pub(crate) fn debug(&self) -> Result<(), Error> {
        write(
            "/home/canfro/instructions.txt",
            format!("{:#?}", self.instructions),
        )?;
        write("/home/canfro/literals.txt", format!("{:#?}", self.literals))?;
        write(
            "/home/canfro/module_tables.txt",
            format!("{:#?}", self.module_tables),
        )?;
        write("/home/canfro/ast.txt", format!("{:#?}", self.registry))?;

        Ok(())
    }

    fn check_module(&mut self, module: &'a Path, line: usize, col: usize) -> Result<(), Error> {
        // Guard for missing module
        let Some(stmts) = self.registry.get(module) else {
            return loc_error(
                line,
                col,
                format!("Module '{}' not found in registry", module.display()).as_str(),
            );
        };

        // Guard for circular import
        if self.checked_modules.contains(&module) {
            return loc_error(
                line,
                col,
                format!(
                    "Circular import between modules '{}' and '{}'",
                    self.current_module.display(),
                    module.display()
                )
                .as_str(),
            );
        }

        // Update current module path
        let old_module = self.current_module;
        self.current_module = module;

        // Mark current module as checked
        self.checked_modules.insert(module);

        // Insert this module's global scope, type, and container maps
        self.module_tables.insert(
            self.current_module,
            ModuleTable {
                scopes: vec![Scope {
                    parent: None,
                    scope_symbols: HashMap::new(),
                }],
                type_symbols: HashMap::new(),
                containers: HashMap::new(),
            },
        );
        self.scope_index = 0;

        // Pass 1 (Global symbol population and import checking)
        for stmt in stmts {
            self.check_stmt_first(stmt)?;
        }

        // Pass 2 (Inner logic and instruction/literal production)
        for stmt in stmts {
            self.check_stmt_second(stmt)?;
        }

        // Restore current module path once checks are over
        self.current_module = old_module;

        Ok(())
    }

    pub(super) fn resolve_module_table(
        &self,
        path: &Path,
        line: usize,
        col: usize,
    ) -> Result<&ModuleTable<'a>, Error> {
        if let Some(module) = self.module_tables.get(path) {
            return Ok(module);
        }

        loc_error(
            line,
            col,
            format!("Module '{}' not found in module registry", path.display()).as_str(),
        )
    }

    pub(super) fn resolve_module_table_mut(
        &mut self,
        path: &Path,
        line: usize,
        col: usize,
    ) -> Result<&mut ModuleTable<'a>, Error> {
        if let Some(module) = self.module_tables.get_mut(path) {
            return Ok(module);
        }

        loc_error(
            line,
            col,
            format!("Module '{}' not found in module registry", path.display()).as_str(),
        )
    }

    pub(super) fn define_scope_symbol(
        &mut self,
        id: &'a str,
        symbol: ScopeSymbol<'a>,
        line: usize,
        col: usize,
    ) -> Result<(), Error> {
        let scope_index = self.scope_index;

        let symbol_ref = self.arena.alloc(symbol);

        let module = self.resolve_module_table_mut(self.current_module, line, col)?;
        let Some(current_scope) = module.scopes.get_mut(scope_index) else {
            return loc_error(
                line,
                col,
                format!(
                    "No scopes found for module '{}'",
                    self.current_module.display()
                )
                .as_str(),
            );
        };

        current_scope.scope_symbols.insert(id, symbol_ref);

        Ok(())
    }

    pub(super) fn define_type_symbol(
        &mut self,
        id: &'a str,
        symbol: TypeSymbol<'a>,
        line: usize,
        col: usize,
    ) -> Result<(), Error> {
        let symbol_ref = self.arena.alloc(symbol);

        let module = self.resolve_module_table_mut(self.current_module, line, col)?;
        module.type_symbols.insert(id, symbol_ref);

        Ok(())
    }

    pub(super) fn define_container(
        &mut self,
        id: &'a str,
        symbol: Container<'a>,
        line: usize,
        col: usize,
    ) -> Result<(), Error> {
        let symbol_ref = self.arena.alloc(symbol);

        let module = self.resolve_module_table_mut(self.current_module, line, col)?;
        module.containers.insert(id, symbol_ref);

        Ok(())
    }

    pub(super) fn resolve_scope_symbol(
        &self,
        id: &str,
        path: &Path,
        line: usize,
        col: usize,
    ) -> Result<&'a ScopeSymbol<'a>, Error> {
        let module = self.resolve_module_table(path, line, col)?;

        if path != self.current_module {
            let Some(symbol) = module.scopes.first().unwrap().scope_symbols.get(id) else {
                return loc_error(
                    line,
                    col,
                    format!(
                        "Global symbol '{}' not found in module '{}'",
                        id,
                        path.display()
                    )
                    .as_str(),
                );
            };

            if matches!(symbol.visibility, Visibility::Priv) {
                return loc_error(line, col, format!("Symbol '{}' is private", id).as_str());
            }

            return Ok(symbol);
        }

        let mut lookup_index = Some(self.scope_index);
        while let Some(idx) = lookup_index {
            let scope = module.scopes.get(idx).unwrap();

            if let Some(symbol) = scope.scope_symbols.get(id) {
                return Ok(symbol);
            };

            lookup_index = scope.parent;
        }

        loc_error(
            line,
            col,
            format!("Local symbol '{}' not found in any scope", id).as_str(),
        )
    }

    pub(super) fn resolve_type_symbol(
        &self,
        id: &str,
        path: &Path,
        line: usize,
        col: usize,
    ) -> Result<&'a TypeSymbol<'a>, Error> {
        let module = self.resolve_module_table(path, line, col)?;
        let Some(symbol) = module.type_symbols.get(id) else {
            return loc_error(
                line,
                col,
                format!(
                    "Type symbol '{}' not found in module '{}'",
                    id,
                    path.display()
                )
                .as_str(),
            );
        };

        if matches!(symbol.visibility, Visibility::Priv) && path != self.current_module {
            return loc_error(line, col, format!("Symbol '{}' is private", id).as_str());
        }

        return Ok(symbol);
    }

    pub(super) fn resolve_container(
        &self,
        id: &str,
        path: &Path,
        line: usize,
        col: usize,
    ) -> Result<&'a Container<'a>, Error> {
        let module = self.resolve_module_table(path, line, col)?;
        let Some(symbol) = module.containers.get(id) else {
            return loc_error(
                line,
                col,
                format!(
                    "Container '{}' not found in module '{}'",
                    id,
                    path.display()
                )
                .as_str(),
            );
        };

        if matches!(symbol.visibility, Visibility::Priv) && path != self.current_module {
            return loc_error(line, col, format!("Container '{}' is private", id).as_str());
        }

        return Ok(symbol);
    }
}
