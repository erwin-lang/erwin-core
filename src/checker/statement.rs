use std::{collections::HashMap, mem::take, path::Path};

use crate::{
    checker::Checker,
    error::Error,
    structure::{
        ast::{
            Expr, ExprKind, Field, Param, Statement, StatementKind, VarKind, Variant, Visibility,
        },
        symbols::{Container, Entry, Symbol},
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
            } => self.define_symbol(
                id,
                Symbol {
                    ty: ty.clone(),
                    visibility,
                    is_static_member: true,
                    is_mutable: false,
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
            } => self.define_symbol(
                id,
                Symbol {
                    ty: ty.clone(),
                    visibility,
                    is_static_member: true,
                    is_mutable: false,
                },
                stmt.line,
                stmt.col,
            ),
            StatementKind::State {
                visibility,
                id,
                fields,
            } => self.check_global_state(stmt, visibility, id, fields),
            StatementKind::Container {
                visibility,
                id,
                types: _,
            } => self.define_container(
                self.current_module,
                id,
                Container {
                    visibility,
                    registry: Vec::new(),
                },
                stmt.line,
                stmt.col,
            ),
            StatementKind::Enum {
                visibility,
                id,
                variants,
            } => self.check_global_enum(stmt, visibility, id, variants),
            StatementKind::Method { id, methods } => self.check_global_method(stmt, id, methods),
            StatementKind::Alias { alias_id, ty } => self.check_global_alias(stmt, alias_id, ty),
            _ => Ok(()),
        }
    }

    pub(super) fn check_statement(&mut self, stmt: &'a Statement<'a>) -> Result<(), Error> {
        match &stmt.kind {
            StatementKind::VarDeclare {
                visibility,
                kind,
                id,
                ty,
                value,
            } => self.check_var(stmt, visibility, kind, id, ty, value),
            StatementKind::VarAssign { id, value } => self.check_assign(stmt, id, value),
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
            StatementKind::Container {
                visibility: _,
                id,
                types,
            } => self.check_container(stmt, id, types),
            StatementKind::Method { id, methods } => self.check_method(stmt, id, methods),
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
        let symbol = Symbol {
            ty: Type::Module(registry_path),
            visibility: &Visibility::Priv,
            is_static_member: true,
            is_mutable: false,
        };

        self.define_symbol(mod_name, symbol, stmt.line, stmt.col)
    }

    fn check_global_state(
        &mut self,
        stmt: &Statement<'a>,
        visibility: &'a Visibility,
        id: &'a str,
        fields: &'a Vec<Field<'a>>,
    ) -> Result<(), Error> {
        if self.current_scopes.len() != 1 {
            return self.loc_error(
                stmt.line,
                stmt.col,
                "State blocks must be defined in the global scope of a module".to_string(),
            );
        }

        let mut entry = Entry {
            ty: Type::Custom { module: None, id },
            visibility,
            symbols: HashMap::new(),
        };

        for field in fields {
            let visibility = if matches!(visibility, Visibility::Priv) {
                &Visibility::Priv
            } else {
                &field.visibility
            };

            let member = Symbol {
                ty: field.ty.clone(),
                visibility,
                is_static_member: false,
                is_mutable: true,
            };

            entry.symbols.insert(field.id, member);
        }

        self.define_entry(self.current_module, id, entry, stmt.line, stmt.col)
    }

    fn check_global_enum(
        &mut self,
        stmt: &Statement<'a>,
        visibility: &'a Visibility,
        id: &'a str,
        variants: &Vec<Variant<'a>>,
    ) -> Result<(), Error> {
        if self.current_scopes.len() != 1 {
            return self.loc_error(
                stmt.line,
                stmt.col,
                "Enums must be defined in the global scope of a module".to_string(),
            );
        }

        let mut entry = Entry {
            ty: Type::Custom { module: None, id },
            visibility,
            symbols: HashMap::new(),
        };

        for variant in variants {
            let variant_ty = if variant.data.is_empty() {
                Type::Custom { module: None, id }
            } else {
                Type::Function {
                    params: variant.data.clone(),
                    return_ty: Box::new(Type::Custom { module: None, id }),
                }
            };

            entry.symbols.insert(
                variant.id,
                Symbol {
                    ty: variant_ty,
                    visibility,
                    is_static_member: true,
                    is_mutable: false,
                },
            );
        }

        self.define_entry(self.current_module, id, entry, stmt.line, stmt.col)
    }

    fn check_global_alias(
        &mut self,
        stmt: &Statement<'a>,
        alias_id: &'a str,
        ty: &'a Type<'a>,
    ) -> Result<(), Error> {
        if alias_id == "Self" {
            return self.loc_error(
                stmt.line,
                stmt.col,
                "'Self' is a reserved type alias".to_string(),
            );
        }

        self.type_aliases.insert(alias_id, ty.clone());

        Ok(())
    }

    fn check_global_method(
        &mut self,
        stmt: &Statement<'a>,
        id: &'a str,
        methods: &'a Expr<'a>,
    ) -> Result<(), Error> {
        let ExprKind::Block(stmts) = &methods.kind else {
            return Ok(());
        };

        let mut targets = Vec::new();

        if self.resolve_entry_mut(id, stmt.line, stmt.col).is_ok() {
            targets.push(id);
        } else if let Ok(container) = self.resolve_container_mut(id, stmt.line, stmt.col) {
            for entry_id in container.registry.clone() {
                targets.push(entry_id);
            }
        } else {
            return self.loc_error(
                stmt.line,
                stmt.col,
                format!("'{}' is neither a type or a container", id),
            );
        }

        for entry_id in targets {
            let entry = self.resolve_entry_mut(entry_id, stmt.line, stmt.col)?;

            for stmt in stmts {
                if let StatementKind::Func {
                    visibility,
                    id: f_id,
                    params,
                    ty,
                    ..
                } = &stmt.kind
                {
                    let is_static = params.first().is_none_or(|p| p.id != "self");
                    let mut final_ty = ty.clone();

                    if !is_static
                        && let Type::Function {
                            params: ty_params, ..
                        } = &mut final_ty
                    {
                        ty_params.insert(0, entry.ty.clone());
                    }

                    entry.symbols.insert(
                        f_id,
                        Symbol {
                            ty: final_ty,
                            visibility,
                            is_static_member: is_static,
                            is_mutable: false,
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
        kind: &VarKind,
        id: &'a str,
        ty: &'a Option<Type<'a>>,
        value: &'a Expr<'a>,
    ) -> Result<(), Error> {
        let val_ty = self.check_expr(stmt, value)?;
        let final_ty = if let Some(explicit_ty) = ty {
            let resolved_ty = self.resolve_alias(explicit_ty);

            if !self.is_assignable(&resolved_ty, &val_ty) {
                return self.loc_error(
                    stmt.line,
                    stmt.col,
                    format!(
                        "Type mismatch: Expected {:?}, found {:?}",
                        resolved_ty, val_ty
                    ),
                );
            }
            resolved_ty
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

        let is_mutable = match kind {
            VarKind::Const => false,
            VarKind::Var => true,
        };

        self.define_symbol(
            id,
            Symbol {
                ty: final_ty,
                visibility,
                is_static_member: self.current_scopes.len() == 1,
                is_mutable,
            },
            stmt.line,
            stmt.col,
        )
    }

    fn check_assign(
        &mut self,
        stmt: &'a Statement<'a>,
        var: &'a Expr<'a>,
        value: &'a Expr<'a>,
    ) -> Result<(), Error> {
        let var_id = match var.kind {
            ExprKind::Identifier(id) => id,
            ExprKind::StaticAccess { member, .. } => member,
            ExprKind::MemberAccess { member, .. } => member,
            _ => {
                return self.loc_error(
                    var.line,
                    var.col,
                    "Symbol does not support assignment".to_string(),
                );
            }
        };

        let value_ty = self.check_expr(stmt, value)?;
        let symbol = self.resolve_symbol(var_id, self.current_module, var.line, var.col)?;

        if !symbol.is_mutable {
            return self.loc_error(var.line, var.col, "Symbol is not mutable".to_string());
        }

        if !self.is_assignable(&symbol.ty, &value_ty) {
            return self.loc_error(
                value.line,
                value.col,
                format!(
                    "Type mismatch: expected '{:?}', found '{:?}'",
                    &symbol.ty, &value_ty
                ),
            );
        }

        Ok(())
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
            if param.id == "self" {
                return self.loc_error(
                    stmt.line,
                    stmt.col,
                    "Regular function cannot have self parameter".to_string(),
                );
            }

            self.define_symbol(
                param.id,
                Symbol {
                    ty: param.ty.clone(),
                    visibility: &Visibility::Priv,
                    is_static_member: false,
                    is_mutable: false,
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

        let expected_ret_ty = match ty {
            Type::Function { return_ty, .. } => &(**return_ty),
            _ => ty,
        };

        if !self.is_assignable(expected_ret_ty, &final_ty) {
            return self.loc_error(
                stmt.line,
                stmt.col,
                format!(
                    "Function '{}' expected type '{:?}' but it's body returned '{:?}'",
                    id, expected_ret_ty, final_ty
                ),
            );
        }

        self.returns = parent_returns;
        self.exit_local_scope(stmt.line, stmt.col)?;

        Ok(())
    }

    fn check_container(
        &mut self,
        stmt: &Statement<'a>,
        id: &'a str,
        body: &Vec<Expr<'a>>,
    ) -> Result<(), Error> {
        if self.current_scopes.len() != 1 {
            return self.loc_error(
                stmt.line,
                stmt.col,
                "Type containers be defined in the global scope of a module".to_string(),
            );
        }

        let mut entries = Vec::new();

        for expr in body {
            match &expr.kind {
                ExprKind::Identifier(ty) => {
                    entries.push(*ty);
                }
                ExprKind::StaticAccess { member, .. } => {
                    entries.push(*member);
                }
                _ => {
                    return self.loc_error(
                        expr.line,
                        expr.col,
                        "Invalid container element".to_string(),
                    );
                }
            }
        }

        for entry in &entries {
            self.resolve_entry(entry, self.current_module, stmt.line, stmt.col)?;
        }

        let container = self.resolve_container_mut(id, stmt.line, stmt.col)?;
        container.registry.extend(entries);

        Ok(())
    }

    fn check_method(
        &mut self,
        stmt: &Statement<'a>,
        id: &'a str,
        methods: &'a Expr<'a>,
    ) -> Result<(), Error> {
        if self.current_scopes.len() != 1 {
            return self.loc_error(
                stmt.line,
                stmt.col,
                "Type containers be defined in the global scope of a module".to_string(),
            );
        }

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
                    let mut target_types = Vec::new();

                    if let Ok(entry) =
                        self.resolve_entry(id, self.current_module, stmt.line, stmt.col)
                    {
                        target_types.push(entry.ty.clone());
                    } else if let Ok(container) =
                        self.resolve_container(id, self.current_module, stmt.line, stmt.col)
                    {
                        for entry_id in container.registry.clone() {
                            let entry = self.resolve_entry(
                                entry_id,
                                self.current_module,
                                stmt.line,
                                stmt.col,
                            )?;

                            target_types.push(entry.ty.clone());
                        }
                    }

                    for target_ty in target_types {
                        self.enter_scope();
                        let self_ty = self.type_aliases.insert("Self", target_ty).unwrap();
                        let parent_returns = take(&mut self.returns);

                        for param in params {
                            let param_ty = match param.id {
                                "self" => &self_ty,
                                _ => &param.ty,
                            };

                            self.define_symbol(
                                param.id,
                                Symbol {
                                    ty: param_ty.clone(),
                                    visibility: &Visibility::Priv,
                                    is_static_member: false,
                                    is_mutable: false,
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

                        let expected_ret_ty = match ty {
                            Type::Function { return_ty, .. } => &(**return_ty),
                            _ => ty,
                        };

                        if !self.is_assignable(expected_ret_ty, &final_ty) {
                            return self.loc_error(
                                stmt.line,
                                stmt.col,
                                format!(
                                    "Method '{}' in type '{:?}' expected type '{:?}' but it's body returned '{:?}'",
                                    f_id, id, expected_ret_ty, final_ty
                                ),
                            );
                        }

                        self.returns = parent_returns;
                        self.type_aliases.remove("Self");
                        self.exit_local_scope(stmt.line, stmt.col)?;
                    }
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
