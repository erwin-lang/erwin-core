use std::{collections::HashMap, mem::take, path::Path};

use crate::{
    checker::Checker,
    error::Error,
    structure::{
        ast::{Expr, ExprKind, Field, Param, Statement, StatementKind, Variant, Visibility},
        symbols::{ScopedSymbol, StaticEntry},
        types::Type,
    },
};

impl<'a> Checker<'a> {
    pub(super) fn check_global_statements(&mut self, stmt: &'a Statement<'a>) -> Result<(), Error> {
        match &stmt.kind {
            StatementKind::Import { alias, path } => self.check_global_import(stmt, alias, path),
            StatementKind::Node {
                visibility,
                id,
                ty,
                value: _,
            } => self.define(
                id,
                ScopedSymbol {
                    ty: ty.clone(),
                    visibility,
                    is_static_member: true,
                },
                stmt.line,
                stmt.col,
            ),
            StatementKind::Func {
                visibility,
                id,
                params: _,
                ty,
                body: _,
            } => self.define(
                id,
                ScopedSymbol {
                    ty: ty.clone(),
                    visibility,
                    is_static_member: true,
                },
                stmt.line,
                stmt.col,
            ),
            StatementKind::State {
                visibility,
                id,
                fields,
            } => self.check_global_state(stmt, visibility, id, fields),
            StatementKind::Enum {
                visibility,
                id,
                variants,
            } => self.check_global_enum(stmt, visibility, id, variants),
            StatementKind::Method { id, methods } => self.check_global_method(stmt, id, methods),
            _ => Ok(()),
        }
    }

    pub(super) fn check_statement(&mut self, stmt: &'a Statement<'a>) -> Result<(), Error> {
        match &stmt.kind {
            StatementKind::Var {
                visibility,
                kind: _,
                id,
                ty,
                value,
            } => self.check_var(stmt, visibility, id, ty, value),
            StatementKind::Node {
                visibility: _,
                id: _,
                ty,
                value,
            } => self.check_node(stmt, ty, value),
            StatementKind::Func {
                visibility: _,
                id,
                params,
                ty,
                body,
            } => self.check_func(stmt, id, params, ty, body),
            StatementKind::Method { id, methods } => self.check_method(id, methods),
            StatementKind::Expr(expr) => self.check_stmt_expr(stmt, expr),
            _ => Ok(()),
        }
    }

    fn check_global_import(
        &mut self,
        stmt: &'a Statement<'a>,
        alias: &'a Option<&str>,
        path: &'a Vec<&str>,
    ) -> Result<(), Error> {
        if self.current_scopes.len() != 1 {
            return self.loc_error(
                stmt.line,
                stmt.col,
                "Module imports must be defined in the global scope of a module".to_string(),
            );
        }

        let mut target_path = match path.first() {
            Some(elem) if *elem == "std" => self
                .std_path
                .parent()
                .unwrap_or(Path::new("."))
                .to_path_buf(),
            Some(_) => self
                .main_module
                .parent()
                .unwrap_or(Path::new("."))
                .to_path_buf(),
            None => unreachable!(),
        };

        for part in path {
            target_path.push(part);
        }

        target_path.set_extension("erw");
        let canonical_path = target_path.canonicalize()?;
        let registry_path = match self.modules.keys().find(|k| **k == canonical_path) {
            Some(p) => p.as_path(),
            None => {
                return self.loc_error(
                    stmt.line,
                    stmt.col,
                    format!("Module {} missing from registry", canonical_path.display()),
                );
            }
        };

        self.check_module(registry_path)?;

        let mod_name = if let Some(name) = alias {
            name
        } else {
            path.last().unwrap()
        };
        let symbol = ScopedSymbol {
            ty: Type::Module(registry_path),
            visibility: &Visibility::Priv,
            is_static_member: true,
        };

        self.define(mod_name, symbol, stmt.line, stmt.col)
    }

    fn check_global_state(
        &mut self,
        stmt: &Statement<'a>,
        visibility: &'a Visibility,
        id: &'a str,
        fields: &'a Vec<Field<'a>>,
    ) -> Result<(), Error> {
        let mut entry = StaticEntry {
            visibility,
            members: HashMap::new(),
        };

        for field in fields {
            let visibility = if matches!(visibility, Visibility::Priv) {
                &Visibility::Priv
            } else {
                &field.visibility
            };

            let member = ScopedSymbol {
                ty: field.ty.clone(),
                visibility,
                is_static_member: true,
            };

            entry.members.insert(field.id, member);
        }

        self.define_static(self.current_module, id, entry, stmt.line, stmt.col)
    }

    fn check_global_enum(
        &mut self,
        stmt: &Statement<'a>,
        visibility: &'a Visibility,
        id: &'a str,
        variants: &Vec<Variant<'a>>,
    ) -> Result<(), Error> {
        let mut entry = StaticEntry {
            visibility,
            members: HashMap::new(),
        };

        for variant in variants {
            let variant_ty = if variant.data.is_empty() {
                Type::Custom(id)
            } else {
                Type::Function {
                    params: variant.data.clone(),
                    return_ty: Box::new(Type::Custom(id)),
                }
            };

            entry.members.insert(
                variant.id,
                ScopedSymbol {
                    ty: variant_ty,
                    visibility,
                    is_static_member: true,
                },
            );
        }

        self.define_static(self.current_module, id, entry, stmt.line, stmt.col)
    }

    pub(super) fn check_global_method(
        &mut self,
        stmt: &Statement<'a>,
        id: &'a str,
        methods: &'a Expr<'a>,
    ) -> Result<(), Error> {
        let entry = self.resolve_static_mut(id, self.current_module, stmt.line, stmt.col)?;

        if let ExprKind::Block(stmts) = &methods.kind {
            for stmt in stmts {
                if let StatementKind::Func {
                    visibility,
                    id: f_id,
                    params,
                    ty,
                    body: _,
                } = &stmt.kind
                {
                    let is_static = params.first().is_none_or(|p| p.id != "self");
                    let mut final_ty = ty.clone();

                    if !is_static
                        && let Type::Function {
                            params: ty_params, ..
                        } = &mut final_ty
                    {
                        ty_params.insert(0, Type::from_str(id));
                    }

                    entry.members.insert(
                        f_id,
                        ScopedSymbol {
                            ty: final_ty,
                            visibility,
                            is_static_member: is_static,
                        },
                    );
                }
            }
        }

        Ok(())
    }

    fn check_var(
        &mut self,
        stmt: &'a Statement<'a>,
        visibility: &'a Visibility,
        id: &'a str,
        ty: &Option<Type<'a>>,
        value: &'a Expr<'a>,
    ) -> Result<(), Error> {
        let val_ty = self.check_expr(stmt, value)?;
        let final_ty = if let Some(explicit_ty) = ty {
            if !self.is_assignable(explicit_ty, &val_ty) {
                return self.loc_error(
                    stmt.line,
                    stmt.col,
                    format!(
                        "Type mismatch: Expected {:?}, found {:?}",
                        explicit_ty, val_ty
                    ),
                );
            }
            explicit_ty.clone()
        } else {
            val_ty
        };

        if matches!(final_ty, Type::Unknown) {
            return self.loc_error(stmt.line, stmt.col, format!("Cannot infer type for variable '{}', please provide an explicit type annotation", id));
        }

        if matches!(final_ty, Type::Done) {
            return self.loc_error(
                stmt.line,
                stmt.col,
                format!("Variable '{}' assigned to a termination signal", id),
            );
        }

        self.define(
            id,
            ScopedSymbol {
                ty: final_ty,
                visibility,
                is_static_member: false,
            },
            stmt.line,
            stmt.col,
        )
    }

    fn check_node(
        &mut self,
        stmt: &'a Statement<'a>,
        ty: &Type<'a>,
        value: &'a Expr<'a>,
    ) -> Result<(), Error> {
        if self.current_scopes.len() != 1 {
            return self.loc_error(
                stmt.line,
                stmt.col,
                "Nodes must be defined in the global scope of a module".to_string(),
            );
        }

        if self.is_literal(&value.kind) {
            return self.loc_error(
                stmt.line,
                stmt.col,
                "Nodes cannot be assigned flat literal values; use a variable or constant"
                    .to_string(),
            );
        }

        let value_ty = Type::Node(Box::new(self.check_expr(stmt, value)?));

        if !self.is_assignable(ty, &value_ty) {
            return self.loc_error(
                stmt.line,
                stmt.col,
                format!("Type mismatch: expected {:?}, found {:?}", ty, value_ty),
            );
        }

        Ok(())
    }

    fn check_func(
        &mut self,
        stmt: &'a Statement<'a>,
        id: &'a str,
        params: &Vec<Param<'a>>,
        ty: &Type<'a>,
        body: &'a Expr<'a>,
    ) -> Result<(), Error> {
        if self.current_scopes.len() != 1 {
            return self.loc_error(
                stmt.line,
                stmt.col,
                "Functions must be defined in the global scope of a module; use lambda expressions for local computations".to_string(),
            );
        }

        self.enter_scope();
        let parent_returns = take(&mut self.returns);

        for param in params {
            self.define(
                param.id,
                ScopedSymbol {
                    ty: param.ty.clone(),
                    visibility: &Visibility::Priv,
                    is_static_member: false,
                },
                stmt.line,
                stmt.col,
            )?;
        }

        let body_ty = self.check_expr(stmt, body)?;
        let returns = take(&mut self.returns);
        let mut final_ty = if matches!(body_ty, Type::Done) {
            Type::Unknown
        } else {
            body_ty
        };

        for ret_ty in &returns {
            final_ty = self.join_ty(&final_ty, ret_ty, stmt.line, stmt.col)?;
        }

        if !self.is_assignable(ty, &final_ty) {
            return self.loc_error(
                stmt.line,
                stmt.col,
                format!(
                    "Function '{}' expected type '{:?}' but it's body returned '{:?}'",
                    id, ty, final_ty
                ),
            );
        }

        self.returns = parent_returns;
        self.exit_local_scope(stmt.line, stmt.col)?;

        Ok(())
    }

    fn check_method(&mut self, id: &'a str, methods: &'a Expr<'a>) -> Result<(), Error> {
        if let ExprKind::Block(stmts) = &methods.kind {
            for stmt in stmts {
                if let StatementKind::Func {
                    id: f_id,
                    params,
                    ty,
                    body,
                    ..
                } = &stmt.kind
                {
                    let is_static = {
                        let entry =
                            self.resolve_static(id, self.current_module, stmt.line, stmt.col)?;

                        if let Some(s) = entry.members.get(f_id) {
                            s.is_static_member
                        } else {
                            return self.loc_error(
                                stmt.line,
                                stmt.col,
                                format!("Symbol '{}' not in registry", f_id),
                            );
                        }
                    };

                    self.enter_scope();

                    if !is_static {
                        self.define(
                            "self",
                            ScopedSymbol {
                                ty: Type::from_str(id),
                                visibility: &Visibility::Priv,
                                is_static_member: false,
                            },
                            stmt.line,
                            stmt.col,
                        )?;
                    }

                    for param in params {
                        self.define(
                            param.id,
                            ScopedSymbol {
                                ty: param.ty.clone(),
                                visibility: &Visibility::Priv,
                                is_static_member: false,
                            },
                            stmt.line,
                            stmt.col,
                        )?;
                    }

                    let body_ty = self.check_expr(stmt, body)?;

                    if !self.is_assignable(ty, &body_ty) {
                        return self.loc_error(
                            stmt.line,
                            stmt.col,
                            format!(
                                "Method '{}' defined for '{}' declared return type {:?} but body results in {:?}",
                                f_id, id, ty, body_ty
                            ),
                        );
                    }

                    self.exit_local_scope(stmt.line, stmt.col)?;
                }
            }
        }

        Ok(())
    }

    fn check_stmt_expr(
        &mut self,
        stmt: &'a Statement<'a>,
        expr: &'a Expr<'a>,
    ) -> Result<(), Error> {
        self.check_expr(stmt, expr)?;
        Ok(())
    }
}
