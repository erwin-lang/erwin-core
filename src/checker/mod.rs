pub(super) mod stmt_first;
pub(super) mod stmt_second;
pub(super) mod stmt_third;

use std::{
    collections::{HashMap, HashSet},
    mem::take,
    path::{Path, PathBuf},
};

use crate::{
    error::{Error, loc_error},
    structure::{
        ast::Statement, container::Container, symbol::ScopeSymbol, type_expr::TypeSymbol,
        types::Type,
    },
};

pub(crate) struct Checker<'a> {
    // Received from Resolver, read-only
    std_path: &'a Path,
    prelude_module: &'a Path,
    main_module: &'a Path,
    registry: &'a HashMap<PathBuf, Vec<Statement<'a>>>, // Module registry, this is what we will walk over and check

    // For checking logic (internal to the checker, the assembler won't know about these)
    current_module: &'a Path,           // Module being currently checked
    checked_modules: HashSet<&'a Path>, // Guard against circular imports
    return_types: Vec<Type<'a>>,        // Return types stack for handling nested lambdas
    type_symbols: HashMap<&'a Path, HashMap<&'a str, TypeSymbol<'a>>>, // Non-scoped type expressions defined by state and enum statements
    containers: HashMap<&'a Path, HashMap<&'a str, Container<'a>>>, // Non-scoped type containers defined by container statements
    scoped_symbols: HashMap<&'a Path, Vec<HashMap<&'a str, ScopeSymbol<'a>>>>, // Scope stack for nested scopes

    // This is what we want to build while we check and pass to the assembler
    instructions: Vec<Instruction>, // Assembler will read these as actions holding metadata (they're basically pre-assembly)
    literals: Vec<Literal>,         // Assembler will read these as non-computed values
}

impl<'a> Checker<'a> {
    pub(crate) fn new(
        std_path: &'a Path,
        prelude_module: &'a Path,
        main_module: &'a Path,
        registry: &'a HashMap<PathBuf, Vec<Statement<'a>>>,
    ) -> Self {
        Self {
            std_path,
            prelude_module,
            main_module,
            registry,

            current_module: main_module,
            checked_modules: HashSet::new(),
            return_types: Vec::new(),
            type_symbols: HashMap::new(),
            containers: HashMap::new(),
            scoped_symbols: HashMap::new(),

            instructions: Vec::new(),
            literals: Vec::new(),
        }
    }

    pub(crate) fn check(&mut self) -> Result<(Vec<Instruction>, Vec<Literal>), Error> {
        self.check_module(self.current_module, 0, 0);

        let instructions = take(&mut self.instructions);
        let literals = take(&mut self.literals);
        Ok((instructions, literals))
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
        self.scoped_symbols.insert(self.current_module, Vec::new());
        self.type_symbols
            .insert(self.current_module, HashMap::new());
        self.containers.insert(self.current_module, HashMap::new());

        // Pass 1 (Import checking and TypeSymbol go first because it's better if they exist before everyone else)
        for stmt in stmts {
            self.check_stmt_first(stmt)?;
        }

        // Pass 2 (Everything else is defined now, but not checked)
        for stmt in stmts {
            self.check_stmt_second(stmt)?;
        }

        // Pass 3 (inner logic, inner ScopedSymbols, Instructions and Literals)
        for stmt in stmts {
            self.check_stmt_third(stmt)?;
        }

        // Restore current module path once checks are over
        self.current_module = old_module;

        Ok(())
    }

    pub(super) fn define_type(
        &mut self,
        id: &'a str,
        symbol: TypeSymbol<'a>,
        line: usize,
        col: usize,
    ) -> Result<(), Error> {
        let Some(table) = self.type_symbols.get_mut(self.current_module) else {
            return loc_error(
                line,
                col,
                format!(
                    "Module '{}' not found in type symbol table",
                    self.current_module.display()
                )
                .as_str(),
            );
        };

        table.insert(id, symbol);

        Ok(())
    }

    pub(super) fn define_symbol(
        &mut self,
        id: &'a str,
        symbol: ScopeSymbol<'a>,
        line: usize,
        col: usize,
    ) -> Result<(), Error> {
        let Some(table) = self.scoped_symbols.get_mut(self.current_module) else {
            return loc_error(
                line,
                col,
                format!(
                    "Module '{}' not found in scoped symbol table",
                    self.current_module.display()
                )
                .as_str(),
            );
        };

        let Some(current_scope) = table.last_mut() else {
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

        current_scope.insert(id, symbol);

        Ok(())
    }
}
