use crate::{
    error::Error,
    parser::Parser,
    syntax::{
        ast::{Expr, Statement, VarKind, Visibility},
        token::TokenKind,
    },
};

impl<'a> Parser<'a> {
    pub(super) fn parse_statement(&mut self) -> Result<Statement<'a>, Error> {
        match self.peek(0)?.kind {
            TokenKind::Pub => self.parse_visibility(),
            TokenKind::Import => self.parse_import(),
            TokenKind::Var => self.parse_var(VarKind::Var, Visibility::Priv),
            TokenKind::Const => self.parse_var(VarKind::Const, Visibility::Priv),
            TokenKind::Node => self.parse_var(VarKind::Node, Visibility::Priv),
            TokenKind::Func => self.parse_func(Visibility::Priv),
            TokenKind::State => self.parse_state(Visibility::Priv),
            TokenKind::Enum => self.parse_enum(Visibility::Priv),
            TokenKind::Method => self.parse_method(),
            _ => self.parse_statement_expr(),
        }
    }

    pub(super) fn parse_import(&mut self) -> Result<Statement<'a>, Error> {
        self.advance()?;
        let path = self.parse_path()?;
        self.consume(TokenKind::Semicolon, "Expected ';'");

        Ok(Statement::Import { path })
    }

    pub(super) fn parse_var(
        &mut self,
        kind: VarKind,
        visibility: Visibility,
    ) -> Result<Statement<'a>, Error> {
        self.advance()?;

        let id = if let TokenKind::Identifier(name) = self.peek(0)?.kind {
            self.advance()?;
            name
        } else {
            return self.error("Expected variable name");
        };

        let ty = if matches!(self.peek(0)?.kind, TokenKind::Colon) {
            self.advance()?;
            Some(self.parse_type()?)
        } else {
            None
        };

        self.consume(TokenKind::Assign, "Expected '='")?;
        let value = self.parse_expr()?;

        if !matches!(value, Expr::Block(_)) {
            self.consume(TokenKind::Semicolon, "Expected ';'")?;
        }

        Ok(Statement::Var {
            visibility,
            kind,
            id,
            ty,
            value,
        })
    }

    pub(super) fn parse_func(&mut self, visibility: Visibility) -> Result<Statement<'a>, Error> {
        self.advance()?;

        let id = if let TokenKind::Identifier(name) = self.peek(0)?.kind {
            self.advance()?;
            name
        } else {
            return self.error("Expected function name");
        };

        self.consume(TokenKind::LParen, "Expected '('")?;

        let params = self.parse_comma_separated(|p| p.parse_param())?;

        self.consume(TokenKind::RParen, "Expected ')'")?;
        self.consume(TokenKind::RArrow, "Expected '->'")?;

        let ty = self.parse_type()?;

        let body_line = self.peek(0)?.line;
        let body_col = self.peek(0)?.column;
        let body = self.parse_expr()?;

        if !matches!(body, Expr::Block(_)) {
            return self.loc_error(
                body_line,
                body_col,
                "Function body must be a block expression",
            );
        }

        Ok(Statement::Func {
            visibility,
            id,
            params,
            ty,
            body,
        })
    }

    pub(super) fn parse_state(&mut self, visibility: Visibility) -> Result<Statement<'a>, Error> {
        self.advance()?;

        let id = if let TokenKind::Identifier(name) = self.peek(0)?.kind {
            self.advance()?;
            name
        } else {
            return self.error("Expected object name");
        };

        self.consume(TokenKind::LBrace, "Expected '{'")?;
        let fields = self.parse_comma_separated(|p| p.parse_field())?;
        self.consume(TokenKind::RBrace, "Expected '}'")?;

        Ok(Statement::State {
            visibility,
            id,
            fields,
        })
    }

    pub(super) fn parse_enum(&mut self, visibility: Visibility) -> Result<Statement<'a>, Error> {
        self.advance()?;

        let id = if let TokenKind::Identifier(name) = self.peek(0)?.kind {
            self.advance()?;
            name
        } else {
            return self.error("Expected enumeration name");
        };

        self.consume(TokenKind::LBrace, "Expected '{'")?;
        let variants = self.parse_comma_separated(|p| p.parse_variant())?;
        self.consume(TokenKind::RBrace, "Expected '}'")?;

        Ok(Statement::Enum {
            visibility,
            id,
            variants,
        })
    }

    pub(super) fn parse_method(&mut self) -> Result<Statement<'a>, Error> {
        self.advance()?;

        let id = if let TokenKind::Identifier(name) = self.peek(0)?.kind {
            self.advance()?;
            name
        } else {
            return self.error("Expected object or enumeration name");
        };

        let methods_line = self.peek(0)?.line;
        let methods_col = self.peek(0)?.column;
        let methods = self.parse_expr()?;

        if let Expr::Block(stmts) = &methods {
            for stmt in stmts {
                if !matches!(
                    stmt,
                    Statement::Func {
                        visibility: _,
                        id: _,
                        params: _,
                        ty: _,
                        body: _
                    }
                ) {
                    return self.loc_error(
                        methods_line,
                        methods_col,
                        "Method statements must be functions",
                    );
                }
            }
        } else {
            return self.loc_error(
                methods_line,
                methods_col,
                "Method body must be a block expression",
            );
        }

        Ok(Statement::Method { id, methods })
    }

    pub(super) fn parse_statement_expr(&mut self) -> Result<Statement<'a>, Error> {
        let expr_line = self.peek(0)?.line;
        let expr_col = self.peek(0)?.column;
        let expr = self.parse_expr()?;

        if matches!(
            expr,
            Expr::Block(_)
                | Expr::For {
                    iter: _,
                    range: _,
                    do_body: _,
                    else_body: _
                }
                | Expr::While {
                    condition: _,
                    do_body: _,
                    else_body: _
                }
                | Expr::If {
                    condition: _,
                    do_body: _,
                    else_body: _
                }
                | Expr::Call { base: _, args: _ }
        ) {
            if !self.is_brace_terminated(&expr) {
                self.consume(TokenKind::Semicolon, "Expected ';'")?;
            }
        } else {
            return self.loc_error(expr_line, expr_col, "This expression cannot be a statement");
        }

        Ok(Statement::Expr(expr))
    }

    pub(super) fn parse_visibility(&mut self) -> Result<Statement<'a>, Error> {
        self.advance()?;
        match self.peek(0)?.kind {
            TokenKind::Var => self.parse_var(VarKind::Var, Visibility::Pub),
            TokenKind::Const => self.parse_var(VarKind::Const, Visibility::Pub),
            TokenKind::Node => self.parse_var(VarKind::Node, Visibility::Pub),
            TokenKind::Func => self.parse_func(Visibility::Pub),
            TokenKind::State => self.parse_state(Visibility::Pub),
            TokenKind::Pub => self.error("Repeated visibility modifier"),
            _ => self.error("Invalid visibility modifier"),
        }
    }
}
