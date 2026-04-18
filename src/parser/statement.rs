use crate::{
    error::Error,
    parser::Parser,
    structure::{
        ast::{ExprKind, Statement, StatementKind, VarKind, Visibility},
        token::TokenKind,
        types::Type,
    },
};

impl<'a> Parser<'a> {
    pub(super) fn parse_statement(&mut self) -> Result<Statement<'a>, Error> {
        match self.peek(0)?.kind {
            TokenKind::Pub => self.parse_visibility(),
            TokenKind::Import => self.parse_import(),
            TokenKind::Var => self.parse_var(VarKind::Var, Visibility::Priv),
            TokenKind::Node => self.parse_node(Visibility::Priv),
            TokenKind::Const => self.parse_var(VarKind::Const, Visibility::Priv),
            TokenKind::Func => self.parse_func(Visibility::Priv),
            TokenKind::State => self.parse_state(Visibility::Priv),
            TokenKind::Enum => self.parse_enum(Visibility::Priv),
            TokenKind::Method => self.parse_method(),
            _ => self.parse_statement_expr(),
        }
    }

    fn parse_import(&mut self) -> Result<Statement<'a>, Error> {
        let start_line = self.peek(0)?.line;
        let start_col = self.peek(0)?.col;

        self.advance()?;

        let alias = if let TokenKind::Identifier(name) = self.peek(0)?.kind
            && matches!(self.peek(1)?.kind, TokenKind::Assign)
        {
            self.advance()?;
            self.consume(TokenKind::Assign, "Expected '='")?;
            Some(name)
        } else {
            None
        };

        let path = self.parse_path()?;
        self.consume(TokenKind::Semicolon, "Expected ';'")?;

        let kind = StatementKind::Import { alias, path };

        Ok(Statement {
            kind,
            line: start_line,
            col: start_col,
        })
    }

    fn parse_var(&mut self, kind: VarKind, visibility: Visibility) -> Result<Statement<'a>, Error> {
        let start_line = self.peek(0)?.line;
        let start_col = self.peek(0)?.col;

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

        if !matches!(value.kind, ExprKind::Block(_)) {
            self.consume(TokenKind::Semicolon, "Expected ';'")?;
        }

        let kind = StatementKind::Var {
            visibility,
            kind,
            id,
            ty,
            value,
        };

        Ok(Statement {
            kind,
            line: start_line,
            col: start_col,
        })
    }

    fn parse_node(&mut self, visibility: Visibility) -> Result<Statement<'a>, Error> {
        let start_line = self.peek(0)?.line;
        let start_col = self.peek(0)?.col;

        self.advance()?;

        let id = if let TokenKind::Identifier(name) = self.peek(0)?.kind {
            self.advance()?;
            name
        } else {
            return self.error("Expected node name");
        };

        self.consume(TokenKind::Colon, "Expected ':'")?;
        let ty = self.parse_type()?;
        self.consume(TokenKind::Assign, "Expected '='")?;
        let value = self.parse_expr()?;

        if !matches!(value.kind, ExprKind::Block(_)) {
            self.consume(TokenKind::Semicolon, "Expected ';'")?;
        }

        let kind = StatementKind::Node {
            visibility,
            id,
            ty,
            value,
        };

        Ok(Statement {
            kind,
            line: start_line,
            col: start_col,
        })
    }

    fn parse_func(&mut self, visibility: Visibility) -> Result<Statement<'a>, Error> {
        let start_line = self.peek(0)?.line;
        let start_col = self.peek(0)?.col;

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

        let ty = Type::Function {
            params: params.iter().map(|p| p.ty.clone()).collect(),
            return_ty: Box::new(self.parse_type()?),
        };

        let body_line = self.peek(0)?.line;
        let body_col = self.peek(0)?.col;
        let body = self.parse_expr()?;

        if !matches!(body.kind, ExprKind::Block(_)) {
            return self.loc_error(
                body_line,
                body_col,
                "Function body must be a block expression",
            );
        }

        let kind = StatementKind::Func {
            visibility,
            id,
            params,
            ty,
            body,
        };

        Ok(Statement {
            kind,
            line: start_line,
            col: start_col,
        })
    }

    fn parse_state(&mut self, visibility: Visibility) -> Result<Statement<'a>, Error> {
        let start_line = self.peek(0)?.line;
        let start_col = self.peek(0)?.col;

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

        let kind = StatementKind::State {
            visibility,
            id,
            fields,
        };

        Ok(Statement {
            kind,
            line: start_line,
            col: start_col,
        })
    }

    fn parse_enum(&mut self, visibility: Visibility) -> Result<Statement<'a>, Error> {
        let start_line = self.peek(0)?.line;
        let start_col = self.peek(0)?.col;

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

        let kind = StatementKind::Enum {
            visibility,
            id,
            variants,
        };

        Ok(Statement {
            kind,
            line: start_line,
            col: start_col,
        })
    }

    fn parse_method(&mut self) -> Result<Statement<'a>, Error> {
        let start_line = self.peek(0)?.line;
        let start_col = self.peek(0)?.col;

        self.advance()?;

        let id = if let TokenKind::Identifier(name) = self.peek(0)?.kind {
            self.advance()?;
            name
        } else {
            return self.error("Expected object or enumeration name");
        };

        let methods_line = self.peek(0)?.line;
        let methods_col = self.peek(0)?.col;
        let methods = self.parse_expr()?;

        if let ExprKind::Block(stmts) = &methods.kind {
            for stmt in stmts {
                if !matches!(stmt.kind, StatementKind::Func { .. }) {
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

        let kind = StatementKind::Method { id, methods };

        Ok(Statement {
            kind,
            line: start_line,
            col: start_col,
        })
    }

    fn parse_statement_expr(&mut self) -> Result<Statement<'a>, Error> {
        let start_line = self.peek(0)?.line;
        let start_col = self.peek(0)?.col;
        let expr = self.parse_expr()?;

        if !matches!(
            expr.kind,
            ExprKind::Block(_)
                | ExprKind::For { .. }
                | ExprKind::While { .. }
                | ExprKind::If { .. }
                | ExprKind::Call { .. }
                | ExprKind::Return(_)
                | ExprKind::Break
                | ExprKind::Continue
                | ExprKind::Yield(_)
                | ExprKind::StaticAccess { .. }
                | ExprKind::MemberAccess { .. }
        ) {
            return self.loc_error(
                start_line,
                start_col,
                "This expression cannot be a statement",
            );
        }

        if !self.is_brace_terminated(&expr) {
            self.consume(TokenKind::Semicolon, "Expected ';'")?;
        }

        let kind = StatementKind::Expr(expr);

        Ok(Statement {
            kind,
            line: start_line,
            col: start_col,
        })
    }

    fn parse_visibility(&mut self) -> Result<Statement<'a>, Error> {
        self.advance()?;
        match self.peek(0)?.kind {
            TokenKind::Var => self.parse_var(VarKind::Var, Visibility::Pub),
            TokenKind::Node => self.parse_node(Visibility::Pub),
            TokenKind::Const => self.parse_var(VarKind::Const, Visibility::Pub),
            TokenKind::Func => self.parse_func(Visibility::Pub),
            TokenKind::State => self.parse_state(Visibility::Pub),
            TokenKind::Pub => self.error("Repeated visibility modifier"),
            _ => self.error("Invalid visibility modifier"),
        }
    }
}
