use crate::{checker::Checker, error::Error, structure::parser::ast::Statement};

impl<'a> Checker<'a> {
    pub(super) fn check_stmt_second(&mut self, stmt: &'a Statement<'a>) -> Result<(), Error> {
        match &stmt.kind {
            _ => Ok(()),
        }
    }
}
