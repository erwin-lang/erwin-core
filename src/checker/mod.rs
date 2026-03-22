use std::collections::HashMap;

use crate::structure::{ast::Statement, symbols::Symbol, types::Type};

pub(crate) struct Checker<'a> {
    pub(crate) statements: Vec<Statement>,
    pub(crate) scopes: Vec<HashMap<&'a str, Symbol<'a>>>,
}

impl<'a> Checker<'a> {
    pub(crate) fn new(statements: Vec<Statement<'a>>) -> Self {
        Self {
            statements,
            scopes: Vec::new().push(HashMap::new()),
        }
    }

    pub(super) fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub(super) fn exit_scope(&mut self) {
        if self.scopes.len() == 1 {
            return self.
        }
    }

    pub(super) fn error(&self) {

    }
}
