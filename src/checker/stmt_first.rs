use std::path::Path;

use crate::{
    checker::Checker,
    error::Error,
    structure::{
        checker::module_table::{ScopeSymbol, Type},
        parser::ast::{Expr, Statement, StatementKind, Visibility},
    },
};

impl<'a> Checker<'a> {
    pub(super) fn check_stmt_first(&mut self, stmt: &'a Statement<'a>) -> Result<(), Error> {
        match &stmt.kind {
            StatementKind::Import {
                alias,
                resolved_path,
                ..
            } => self.check_import(stmt, alias, resolved_path),
            StatementKind::VarDeclare {
                visibility,
                kind,
                id,
                ty,
                ..
            } => self.check_var_declare(stmt, visibility, id, ty),
            StatementKind::Node {
                visibility,
                id,
                ty,
                value,
            } => {}
            _ => Ok(()),
        }
    }

    fn check_import(
        &mut self,
        stmt: &Statement<'a>,
        alias: &'a Option<&str>,
        resolved_path: &'a Path,
    ) -> Result<(), Error> {
        self.check_module(resolved_path, stmt.line, stmt.col)?;
        let id = alias.unwrap_or(resolved_path.file_stem().unwrap().to_str().unwrap());

        self.define_scope_symbol(
            id,
            ScopeSymbol {
                id,
                visibility: &Visibility::Priv,
                ty: Type::Module(resolved_path),
                is_static_member: true,
            },
            stmt.line,
            stmt.col,
        )
    }

    fn check_var_declare(
        &mut self,
        stmt: &Statement<'a>,
        visibility: &'a Visibility,
        id: &'a str,
        ty: &Option<Expr<'a>>,
    ) -> Result<(), Error> {
        let final_ty = match ty {
            Some(e) => self.check_type(e)?,
            None => Type::Unknown,
        };

        self.define_scope_symbol(
            id,
            ScopeSymbol {
                id,
                visibility,
                ty: final_ty,
                is_static_member: self.scoped_symbols.get(self.current_module).unwrap().len() == 1,
            },
            stmt.line,
            stmt.col,
        )
    }
}
