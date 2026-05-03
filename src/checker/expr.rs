use std::path::Path;

use crate::{
    checker::Checker,
    error::{Error, loc_error},
    structure::{
        checker::{
            literal::Literal,
            module_table::{Sign, Type},
            resolved_expr::ResolvedExpr,
        },
        parser::ast::{Expr, ExprKind},
    },
};

impl<'a> Checker<'a> {
    pub(super) fn check_expr(&mut self, expr: &'a Expr<'a>) -> Result<ResolvedExpr<'a>, Error> {
        match &expr.kind {
            ExprKind::Number(num) => self.check_num(expr, num, Sign::Unsigned),
            ExprKind::String(s) => Ok(ResolvedExpr::Literal(Literal::Str(s))),
            ExprKind::Bool(b) => Ok(ResolvedExpr::Literal(Literal::Bool(b))),
            ExprKind::Identifier(id) => self.check_identifier(expr, id, self.current_module),
            ExprKind::StaticAccess { target, member } => self.check_static_access(target, member),
            ExprKind::MemberAccess { target, member } => self.check_member_access(target, member),
            _ => todo!(),
        }
    }

    fn check_num(
        &mut self,
        expr: &Expr<'a>,
        num: &str,
        sign: Sign,
    ) -> Result<ResolvedExpr<'a>, Error> {
        if num.contains('.') {
            let val = num.parse::<f64>().or_else(|_| {
                loc_error(
                    expr.line,
                    expr.col,
                    "Floating point number is out of bounds",
                )
            })?;

            let lit = if val as f32 as f64 == val {
                Literal::Float32(val as f32)
            } else {
                Literal::Float64(val)
            };

            return Ok(ResolvedExpr::Literal(lit));
        }

        match sign {
            Sign::Unsigned => {
                let val = num.parse::<u128>().or_else(|_| {
                    loc_error(expr.line, expr.col, "Unsigned integer is out of bounds")
                })?;

                let lit = if val <= u8::MAX as u128 {
                    Literal::UInt8(val as u8)
                } else if val <= u16::MAX as u128 {
                    Literal::UInt16(val as u16)
                } else if val <= u32::MAX as u128 {
                    Literal::UInt32(val as u32)
                } else if val <= u64::MAX as u128 {
                    Literal::UInt64(val as u64)
                } else {
                    Literal::UInt128(val)
                };

                return Ok(ResolvedExpr::Literal(lit));
            }
            Sign::Signed => {
                let val = num.parse::<i128>().or_else(|_| {
                    loc_error(expr.line, expr.col, "Signed integer is out of bounds")
                })?;

                let lit = if val <= (i8::MIN as i128).abs() {
                    Literal::Int8(val as i8)
                } else if val <= (i16::MIN as i128).abs() {
                    Literal::Int16(val as i16)
                } else if val <= (i32::MIN as i128).abs() {
                    Literal::Int32(val as i32)
                } else if val <= (i64::MIN as i128).abs() {
                    Literal::Int64(val as i64)
                } else {
                    Literal::Int128(val)
                };

                return Ok(ResolvedExpr::Literal(lit));
            }
        }
    }

    fn check_identifier(
        &mut self,
        expr: &Expr<'a>,
        id: &str,
        path: &Path,
    ) -> Result<ResolvedExpr<'a>, Error> {
        if let Ok(symbol) = self.resolve_scope_symbol(id, path, expr.line, expr.col) {
            return Ok(ResolvedExpr::ScopeSymbol(symbol));
        } else if let Ok(type_symbol) = self.resolve_type_symbol(id, path, expr.line, expr.col) {
            // If a Type and TypeSymbol are equal (for example, custom types) they will both fall
            // here; self.check_type should differentiate the return
            return Ok(ResolvedExpr::TypeSymbol(type_symbol));
        } else if let Ok(container) = self.resolve_container(id, path, expr.line, expr.col) {
            return Ok(ResolvedExpr::Container(container));
        }

        loc_error(
            expr.line,
            expr.col,
            format!("Undefined identifier '{}'", id).as_str(),
        )
    }

    fn check_static_access(
        &mut self,
        target: &'a Expr<'a>,
        member: &str,
    ) -> Result<ResolvedExpr<'a>, Error> {
        let resolved_target = self.check_expr(target)?;

        match resolved_target {
            ResolvedExpr::ScopeSymbol(s) => {
                let Type::Module(path) = s.ty else {
                    return loc_error(
                        target.line,
                        target.col,
                        "Target scoped symbol can only be a module",
                    );
                };

                self.check_identifier(target, member, path)
            }
            ResolvedExpr::TypeSymbol(t) => {
                let Some(symbol) = t.members.get(member) else {
                    return loc_error(
                        target.line,
                        target.col,
                        format!("No member '{}' in type symbol '{:?}'", member, t).as_str(),
                    );
                };

                if !symbol.is_static_member {
                    return loc_error(
                        target.line,
                        target.col,
                        format!("Member '{}' of type symbol '{:?}' is not static", member, t)
                            .as_str(),
                    );
                }

                Ok(ResolvedExpr::ScopeSymbol(symbol))
            }
            _ => loc_error(
                target.line,
                target.col,
                "Target doesn't allow static access",
            ),
        }
    }

    fn check_member_access(
        &mut self,
        target: &'a Expr<'a>,
        member: &str,
    ) -> Result<ResolvedExpr<'a>, Error> {
        let resolved_target = self.check_expr(target)?;
        let type_symbol_id = match resolved_target {
            ResolvedExpr::ScopeSymbol(s) => s.ty.type_symbol_id().unwrap().as_str(),
            ResolvedExpr::Literal(lit) => lit.type_symbol_id().as_str(),
            _ => {
                return loc_error(
                    target.line,
                    target.col,
                    "Target doesn't allow member access",
                );
            }
        };
        let type_symbol =
            self.resolve_type_symbol(type_symbol_id, self.current_module, target.line, target.col)?;
        let Some(symbol) = type_symbol.members.get(member) else {
            return loc_error(
                target.line,
                target.col,
                format!("No member '{}' in type symbol '{:?}'", member, type_symbol).as_str(),
            );
        };

        if symbol.is_static_member {
            return loc_error(
                target.line,
                target.col,
                format!(
                    "Member '{}' of type symbol '{:?}' is static",
                    member, type_symbol
                )
                .as_str(),
            );
        }

        Ok(ResolvedExpr::ScopeSymbol(symbol))
    }
}
