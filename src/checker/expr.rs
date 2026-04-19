use std::mem::take;

use crate::{
    checker::Checker,
    error::Error,
    structure::{
        ast::{
            BinaryOp, Expr, ExprKind, InstanceField, Param, Statement, StatementKind, UnaryOp,
            Visibility,
        },
        symbols::ScopedSymbol,
        types::{FloatSize, IntSize, Sign, Type},
    },
};

impl<'a> Checker<'a> {
    pub(super) fn check_expr(
        &mut self,
        stmt: &'a Statement<'a>,
        expr: &'a Expr<'a>,
    ) -> Result<Type<'a>, Error> {
        match &expr.kind {
            ExprKind::Number(num) => self.check_number(expr, num, Sign::Unsigned),
            ExprKind::String(_) => Ok(Type::String),
            ExprKind::Bool(_) => Ok(Type::Bool),
            ExprKind::Break => Ok(Type::Done),
            ExprKind::Continue => Ok(Type::Done),
            ExprKind::Return(ret) => self.check_return(stmt, ret),
            ExprKind::Yield(_) => Ok(Type::Unknown),
            ExprKind::Identifier(id) => self.check_identifier(expr, id),
            ExprKind::MemberAccess { target, member } => {
                self.check_member_access(stmt, target, member)
            }
            ExprKind::StaticAccess { target, member } => {
                self.check_static_access(stmt, target, member)
            }
            ExprKind::Tuple(vec) => self.check_tuple(stmt, vec),
            ExprKind::Array(vec) => self.check_array(stmt, vec),
            ExprKind::Block(vec) => self.check_block(expr, vec),
            ExprKind::StateInstance { id, fields } => {
                self.check_state_instance(stmt, expr, id, fields)
            }
            ExprKind::Call { base, args } => self.check_call(stmt, expr, base, args),
            ExprKind::Unary { op, right } => self.check_unary(stmt, op, right),
            ExprKind::Binary { left, op, right } => self.check_binary(stmt, expr, left, op, right),
            ExprKind::For {
                elem,
                iter,
                do_body,
                else_body,
            } => self.check_for(stmt, expr, elem, iter, do_body, else_body),
            ExprKind::While {
                condition,
                do_body,
                else_body,
            } => self.check_conditional(stmt, expr, condition, do_body, else_body),
            ExprKind::If {
                condition,
                do_body,
                else_body,
            } => self.check_conditional(stmt, expr, condition, do_body, else_body),
            ExprKind::Lambda { params, body } => self.check_lambda(stmt, expr, params, body),
        }
    }

    fn check_number(&self, expr: &Expr<'a>, num: &str, sign: Sign) -> Result<Type<'a>, Error> {
        if num.contains('.') {
            let f64_val = num.parse::<f64>().or_else(|_| {
                self.loc_error(
                    expr.line,
                    expr.col,
                    "Float exceeds maximum possible byte size".to_string(),
                )
            })?;
            let f32_val = f64_val as f32;

            if f32_val as f64 == f64_val {
                return Ok(Type::Float {
                    size: FloatSize::B32,
                });
            } else {
                return Ok(Type::Float {
                    size: FloatSize::B64,
                });
            }
        }

        let size = match sign {
            Sign::Unsigned => {
                let u128_val = num.parse::<u128>().or_else(|_| {
                    self.loc_error(
                        expr.line,
                        expr.col,
                        "Integer exceeds maximum possible byte size".to_string(),
                    )
                })?;

                match u128_val {
                    v if v <= u8::MAX as u128 => IntSize::B8,
                    v if v <= u16::MAX as u128 => IntSize::B16,
                    v if v <= u32::MAX as u128 => IntSize::B32,
                    v if v <= u64::MAX as u128 => IntSize::B64,
                    _ => IntSize::B128,
                }
            }
            Sign::Signed => {
                let i128_val = num.parse::<i128>().or_else(|_| {
                    self.loc_error(
                        expr.line,
                        expr.col,
                        "Integer exceeds maximum possible byte size".to_string(),
                    )
                })?;

                match i128_val {
                    v if v <= (i8::MIN as i128).abs() => IntSize::B8,
                    v if v <= (i16::MIN as i128).abs() => IntSize::B16,
                    v if v <= (i32::MIN as i128).abs() => IntSize::B32,
                    v if v <= (i64::MIN as i128).abs() => IntSize::B64,
                    _ => IntSize::B128,
                }
            }
        };

        Ok(Type::Integer { size, sign })
    }

    fn check_return(
        &mut self,
        stmt: &'a Statement<'a>,
        ret: &'a Expr<'a>,
    ) -> Result<Type<'a>, Error> {
        let ty = self.check_expr(stmt, ret)?;
        self.returns.push(ty);

        Ok(Type::Done)
    }

    fn check_identifier(&self, expr: &Expr<'a>, id: &'a str) -> Result<Type<'a>, Error> {
        if let Some(s) = self.resolve(id) {
            return Ok(s.ty.clone());
        }

        if self
            .resolve_static(id, self.current_module, expr.line, expr.col)
            .is_ok()
        {
            return Ok(Type::from_str(id));
        }

        if self
            .resolve_static(id, self.prelude_module, expr.line, expr.col)
            .is_ok()
        {
            return Ok(Type::from_str(id));
        }

        self.loc_error(
            expr.line,
            expr.col,
            format!("Undefined identifier '{}'", id),
        )
    }

    fn check_member_access(
        &mut self,
        stmt: &'a Statement<'a>,
        target: &'a Expr<'a>,
        member: &str,
    ) -> Result<Type<'a>, Error> {
        let target_ty = self.check_expr(stmt, target)?;

        if matches!(target_ty, Type::Module(_)) {
            return self.loc_error(
                target.line,
                target.col,
                "Imported modules only have static members, use '::'".to_string(),
            );
        }

        let registry_id = target_ty.as_str();

        if let ExprKind::Identifier(id) = target.kind
            && id == registry_id
        {
            return self.loc_error(
                target.line,
                target.col,
                format!(
                    "Type '{}' does not have instance members as it is static. Use '::'",
                    id
                ),
            );
        }

        let entry =
            self.resolve_static(registry_id, self.current_module, target.line, target.col)?;

        if let Some(symbol) = entry.members.get(member) {
            if symbol.is_static_member {
                return self.loc_error(
                    target.line,
                    target.col,
                    format!(
                        "Accessed symbol '{}' is static and can only be accessed directly through the type with '::'",
                        member
                    ),
                );
            }

            Ok(symbol.ty.clone())
        } else {
            self.loc_error(
                target.line,
                target.col,
                format!("Type '{}' has no member '{}'", registry_id, member),
            )
        }
    }

    fn check_static_access(
        &mut self,
        stmt: &'a Statement<'a>,
        target: &'a Expr<'a>,
        member: &str,
    ) -> Result<Type<'a>, Error> {
        let target_ty = self.check_expr(stmt, target)?;

        if let Type::Module(path) = target_ty {
            let symbol = self.resolve_external(member, path, target.line, target.col)?;
            return Ok(symbol.ty.clone());
        }

        let registry_id = target_ty.as_str();

        if let ExprKind::Identifier(id) = target.kind
            && id != registry_id
        {
            return self.loc_error(
                target.line,
                target.col,
                format!(
                    "Symbol '{}' is not a type and doesn't support static access. Use '.' for member access",
                    id
                ),
            );
        }

        let entry =
            self.resolve_static(registry_id, self.current_module, target.line, target.col)?;

        if let Some(symbol) = entry.members.get(member) {
            if !symbol.is_static_member {
                return self.loc_error(
                    target.line,
                    target.col,
                    format!(
                        "Accessed symbol '{}' is not static and can only be accessed through an instance with '.'",
                        member
                    ),
                );
            }

            Ok(symbol.ty.clone())
        } else {
            self.loc_error(
                target.line,
                target.col,
                format!("Type '{}' has no member '{}'", registry_id, member),
            )
        }
    }

    fn check_tuple(
        &mut self,
        stmt: &'a Statement<'a>,
        vec: &'a Vec<Expr<'a>>,
    ) -> Result<Type<'a>, Error> {
        let types: Result<Vec<Type<'a>>, Error> =
            vec.iter().map(|e| self.check_expr(stmt, e)).collect();
        Ok(Type::Tuple(types?))
    }

    fn check_array(
        &mut self,
        stmt: &'a Statement<'a>,
        elems: &'a Vec<Expr<'a>>,
    ) -> Result<Type<'a>, Error> {
        if elems.is_empty() {
            return Ok(Type::Array(Box::new(Type::Unknown)));
        }

        let first_ty = self.check_expr(stmt, &elems[0])?;

        for (i, elem) in elems.iter().enumerate().skip(1) {
            let current_ty = self.check_expr(stmt, elem)?;

            if !self.is_assignable(&first_ty, &current_ty) {
                return self.loc_error(
                    elem.line,
                    elem.col,
                    format!(
                        "Array element at index {} has type '{:?}', but the array expected '{:?}'",
                        i, current_ty, first_ty
                    ),
                );
            }
        }

        Ok(Type::Array(Box::new(first_ty)))
    }

    fn check_block(
        &mut self,
        expr: &Expr<'a>,
        stmts: &'a Vec<Statement<'a>>,
    ) -> Result<Type<'a>, Error> {
        self.enter_scope();
        let mut block_ty = Type::Unit;

        for (i, stmt) in stmts.iter().enumerate() {
            self.check_global_statements(stmt)?;
            self.check_statement(stmt)?;

            if let StatementKind::Expr(e) = &stmt.kind {
                let expr_ty = self.check_expr(stmt, e)?;

                if matches!(expr_ty, Type::Done) {
                    block_ty = Type::Done;
                    break;
                }

                if i == stmts.len() - 1
                    && let ExprKind::Yield(y) = &e.kind
                {
                    block_ty = self.check_expr(stmt, y)?;
                }
            }
        }

        self.exit_local_scope(expr.line, expr.col)?;

        Ok(block_ty)
    }

    fn check_state_instance(
        &mut self,
        stmt: &'a Statement<'a>,
        expr: &Expr<'a>,
        id: &'a str,
        fields: &'a Vec<InstanceField<'a>>,
    ) -> Result<Type<'a>, Error> {
        let mut members = self
            .resolve_static(id, self.current_module, expr.line, expr.col)?
            .members
            .clone();

        for field in fields {
            if let Some(member) = members.remove(field.id) {
                let provided_ty = self.check_expr(stmt, &field.value)?;

                if !self.is_assignable(&member.ty, &provided_ty) {
                    return self.loc_error(
                        field.value.line,
                        field.value.col,
                        format!(
                            "Field '{}' expected type '{:?}', found '{:?}' instead",
                            field.id, member.ty, provided_ty
                        ),
                    );
                }
            } else {
                return self.loc_error(
                    field.value.line,
                    field.value.col,
                    format!("Type '{}' has no field '{}'", id, field.id),
                );
            }
        }

        if !members.is_empty() {
            let name = *members.keys().next().unwrap();
            return self.loc_error(expr.line, expr.col, format!("Missing field '{}'", name));
        }

        Ok(Type::from_str(id))
    }

    fn check_call(
        &mut self,
        stmt: &'a Statement<'a>,
        expr: &Expr<'a>,
        base: &'a Expr<'a>,
        args: &'a Vec<Expr<'a>>,
    ) -> Result<Type<'a>, Error> {
        let base_ty = self.check_expr(stmt, base)?;

        if let Type::Function { params, return_ty } = &base_ty {
            let mut param_types = params.iter();

            for arg in args {
                let param_ty = if let Some(p) = param_types.next() {
                    p
                } else {
                    return self.loc_error(
                        expr.line,
                        expr.col,
                        format!(
                            "Function expected {} arguments, found {}",
                            params.len(),
                            args.len()
                        ),
                    );
                };
                let arg_ty = self.check_expr(stmt, arg)?;

                if !self.is_assignable(param_ty, &arg_ty) {
                    return self.loc_error(
                        arg.line,
                        arg.col,
                        format!("Expected type '{:?}', found '{:?}'", param_ty, &arg_ty),
                    );
                }
            }

            if param_types.next().is_some() {
                return self.loc_error(
                    expr.line,
                    expr.col,
                    format!(
                        "Function expected {} arguments, found {}",
                        params.len(),
                        args.len()
                    ),
                );
            }

            return Ok(*return_ty.clone());
        }

        self.loc_error(
            expr.line,
            expr.col,
            "Cannot call a non-function type".to_string(),
        )
    }

    fn check_unary(
        &mut self,
        stmt: &'a Statement<'a>,
        op: &UnaryOp,
        right: &'a Expr<'a>,
    ) -> Result<Type<'a>, Error> {
        if matches!(op, UnaryOp::Minus)
            && let ExprKind::Number(num) = &right.kind
        {
            return self.check_number(right, num, Sign::Signed);
        }

        let right_ty = self.check_expr(stmt, right)?;

        match op {
            UnaryOp::Not => {
                if !matches!(right_ty, Type::Bool) {
                    return self.loc_error(
                        right.line,
                        right.col,
                        "The not operator requires expression of type 'Bool'".to_string(),
                    );
                }
                Ok(Type::Bool)
            }
            UnaryOp::Minus => {
                if !matches!(right_ty, Type::Integer { .. } | Type::Float { .. }) {
                    return self.loc_error(
                        right.line,
                        right.col,
                        "Minus operator requires numeric type".to_string(),
                    );
                }
                Ok(right_ty)
            }
            UnaryOp::Ref => Ok(Type::Ref(Box::new(right_ty))),
            UnaryOp::Deref => match right_ty {
                Type::Ref(inner) | Type::Pointer(inner) => Ok(*inner),
                _ => self.loc_error(
                    right.line,
                    right.col,
                    "Cannot dereference non-pointer/ref type".to_string(),
                ),
            },
        }
    }

    fn check_binary(
        &mut self,
        stmt: &'a Statement<'a>,
        expr: &Expr<'a>,
        left: &'a Expr<'a>,
        op: &BinaryOp,
        right: &'a Expr<'a>,
    ) -> Result<Type<'a>, Error> {
        let left_ty = self.check_expr(stmt, left)?;
        let right_ty = self.check_expr(stmt, right)?;

        match op {
            BinaryOp::Pow | BinaryOp::Mult | BinaryOp::Div | BinaryOp::Add | BinaryOp::Sub => {
                if matches!(left_ty, Type::Integer { .. } | Type::Float { .. })
                    && matches!(right_ty, Type::Integer { .. } | Type::Float { .. })
                {
                    return self.join_ty(&left_ty, &right_ty, expr.line, expr.col);
                }
                self.loc_error(
                    expr.line,
                    expr.col,
                    "Both sides of the expression must be numeric".to_string(),
                )
            }
            BinaryOp::Range => {
                if let Type::Integer {
                    size: l_size,
                    sign: l_sign,
                } = &left_ty
                    && let Type::Integer {
                        size: r_size,
                        sign: r_sign,
                    } = &right_ty
                {
                    if self.is_assignable(&left_ty, &right_ty) {
                        return Ok(Type::IntRange {
                            size: *l_size,
                            sign: *l_sign,
                        });
                    } else if self.is_assignable(&right_ty, &left_ty) {
                        return Ok(Type::IntRange {
                            size: *r_size,
                            sign: *r_sign,
                        });
                    }
                }

                self.loc_error(
                    expr.line,
                    expr.col,
                    "Range bounds must be integers".to_string(),
                )
            }
            BinaryOp::LessThan
            | BinaryOp::GreaterThan
            | BinaryOp::LessEqual
            | BinaryOp::GreaterEqual => {
                if !matches!(left_ty, Type::Integer { .. } | Type::Float { .. })
                    | !matches!(right_ty, Type::Integer { .. } | Type::Float { .. })
                {
                    return self.loc_error(
                        expr.line,
                        expr.col,
                        "Both sides of the expression must be numeric".to_string(),
                    );
                }
                Ok(Type::Bool)
            }
            BinaryOp::Equal | BinaryOp::NotEqual => {
                if !self.is_assignable(&left_ty, &right_ty)
                    && !self.is_assignable(&right_ty, &left_ty)
                {
                    return self.loc_error(
                        expr.line,
                        expr.col,
                        "Incompatible types for comparison".to_string(),
                    );
                }
                Ok(Type::Bool)
            }
            BinaryOp::Or
            | BinaryOp::And
            | BinaryOp::Nor
            | BinaryOp::Nand
            | BinaryOp::Xor
            | BinaryOp::Xnor => {
                if matches!(&left_ty, Type::Bool) && matches!(&right_ty, Type::Bool) {
                    return Ok(Type::Bool);
                } else if matches!(left_ty, Type::Integer { .. })
                    && matches!(right_ty, Type::Integer { .. })
                {
                    return self.join_ty(&left_ty, &right_ty, expr.line, expr.col);
                }
                self.loc_error(
                    expr.line,
                    expr.col,
                    "Logical/Bitwise operators require Bool or Integer types".to_string(),
                )
            }
            BinaryOp::RPipe => {
                let args = match &left.kind {
                    ExprKind::Tuple(vec) => vec.clone(),
                    _ => vec![left.clone()],
                };
                self.check_call(stmt, expr, right, Box::leak(Box::new(args)))
            }
        }
    }

    fn check_for(
        &mut self,
        stmt: &'a Statement<'a>,
        expr: &Expr<'a>,
        elem: &'a str,
        iter: &'a Expr<'a>,
        do_body: &'a Expr<'a>,
        else_body: &'a Option<Box<Expr<'a>>>,
    ) -> Result<Type<'a>, Error> {
        let iter_ty = self.check_expr(stmt, iter)?;
        let elem_ty = match iter_ty.elem_type() {
            Some(e) => e,
            None => return self.loc_error(iter.line, iter.col, "Type is not iterable".to_string()),
        };

        self.enter_scope();
        self.define(
            elem,
            ScopedSymbol {
                ty: elem_ty,
                visibility: &Visibility::Priv,
                is_static_member: false,
                is_mutable: false,
            },
            expr.line,
            expr.col,
        )?;

        let do_ty = self.check_expr(stmt, do_body)?;
        self.exit_local_scope(do_body.line, do_body.col)?;
        self.check_do_else(stmt, expr, do_ty, else_body)
    }

    fn check_conditional(
        &mut self,
        stmt: &'a Statement<'a>,
        expr: &'a Expr<'a>,
        condition: &'a Expr<'a>,
        do_body: &'a Expr<'a>,
        else_body: &'a Option<Box<Expr<'a>>>,
    ) -> Result<Type<'a>, Error> {
        let cond_ty = self.check_expr(stmt, condition)?;

        if !matches!(cond_ty, Type::Bool) {
            return self.loc_error(expr.line, expr.col, "Condition must be Bool".to_string());
        }

        let do_ty = self.check_expr(stmt, do_body)?;
        self.check_do_else(stmt, expr, do_ty, else_body)
    }

    fn check_do_else(
        &mut self,
        stmt: &'a Statement<'a>,
        expr: &Expr<'a>,
        do_ty: Type<'a>,
        else_body: &'a Option<Box<Expr<'a>>>,
    ) -> Result<Type<'a>, Error> {
        if let Some(e) = else_body {
            let else_ty = self.check_expr(stmt, e)?;

            return self.join_ty(&do_ty, &else_ty, e.line, e.col);
        }

        if !matches!(stmt.kind, StatementKind::Expr(_)) {
            return self.loc_error(
                expr.line,
                expr.col,
                "Control flow block must cover all branches when used as expression".to_string(),
            );
        }

        Ok(do_ty)
    }

    fn check_lambda(
        &mut self,
        stmt: &'a Statement<'a>,
        expr: &Expr<'a>,
        params: &'a Vec<Param<'a>>,
        body: &'a Expr<'a>,
    ) -> Result<Type<'a>, Error> {
        self.enter_scope();
        let parent_returns = take(&mut self.returns);

        for param in params {
            self.define(
                param.id,
                ScopedSymbol {
                    ty: param.ty.clone(),
                    visibility: &Visibility::Priv,
                    is_static_member: false,
                    is_mutable: false,
                },
                expr.line,
                expr.col,
            )?;
        }

        let body_ty = self.check_expr(stmt, body)?;
        let returns = take(&mut self.returns);
        let mut ty = if matches!(body_ty, Type::Done) {
            Type::Unknown
        } else {
            body_ty
        };

        for ret_ty in &returns {
            ty = self.join_ty(&ty, ret_ty, expr.line, expr.col)?;
        }

        self.returns = parent_returns;
        self.exit_local_scope(expr.line, expr.col)?;

        Ok(Type::Function {
            params: params.iter().map(|p| p.ty.clone()).collect(),
            return_ty: Box::new(ty),
        })
    }
}
