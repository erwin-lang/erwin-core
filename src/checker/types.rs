use crate::{
    checker::Checker,
    error::Error,
    structure::{checker::module_table::Type, parser::ast::Expr},
};

impl<'a> Checker<'a> {
    pub(super) fn check_type(&mut self, expr: &Expr<'a>) -> Result<Type<'a>, Error> {}
}
