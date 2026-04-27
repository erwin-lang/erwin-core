use std::path::Path;

use crate::{
    checker::Checker,
    error::Error,
    structure::{
        ast::{Statement, StatementKind, Visibility},
        symbol::ScopeSymbol,
        type_expr::{TypeSymbol, TypeSymbolKind},
        types::Type,
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
            StatementKind::State { visibility, id, .. } => self.define_type(
                id,
                TypeSymbol {
                    visibility,
                    kind: TypeSymbolKind::from_str(id),
                    members: Vec::new(),
                },
                stmt.line,
                stmt.col,
            ),
            StatementKind::Enum { visibility, id, .. } => self.define_type(
                id,
                TypeSymbol {
                    visibility,
                    kind: TypeSymbolKind::from_str(id),
                    members: Vec::new(),
                },
                stmt.line,
                stmt.col,
            ),
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

        self.define_symbol(
            id,
            ScopeSymbol {
                visibility: &Visibility::Priv,
                ty: Type::Module(resolved_path),
            },
            stmt.line,
            stmt.col,
        )
    }
}
