use std::path::PathBuf;

use crate::{
    error::{Error, loc_error},
    parser::Parser,
    structure::{
        ast::{ExprKind, Statement, StatementKind, VarKind, Visibility},
        token::TokenKind,
    },
};

impl<'a> Parser<'a> {
    pub(super) fn parse_statement(&mut self) -> Result<Statement<'a>, Error> {
        match self.peek(0)?.kind {
            TokenKind::Pub => self.parse_visibility(),
            TokenKind::Import => self.parse_import(),
            TokenKind::Var => self.parse_var(VarKind::Var, Visibility::Priv),
            TokenKind::Mut => self.parse_assign(),
            TokenKind::NodeStmt => self.parse_node(Visibility::Priv),
            TokenKind::Const => self.parse_var(VarKind::Const, Visibility::Priv),
            TokenKind::FuncStmt => self.parse_func(Visibility::Priv),
            TokenKind::State => self.parse_state(Visibility::Priv),
            TokenKind::Container => self.parse_container(Visibility::Priv),
            TokenKind::Enum => self.parse_enum(Visibility::Priv),
            TokenKind::Method => self.parse_method(),
            TokenKind::Alias => self.parse_alias(),
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

        let path = self.parse_expr()?;
        self.consume(TokenKind::Semicolon, "Expected ';'")?;

        Ok(Statement {
            kind: StatementKind::Import {
                alias,
                path,
                resolved_path: PathBuf::new(),
            },
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
            Some(self.parse_expr()?)
        } else {
            None
        };

        self.consume(TokenKind::Assign, "Expected '='")?;
        let value = self.parse_expr()?;

        if !matches!(value.kind, ExprKind::Block(_)) {
            self.consume(TokenKind::Semicolon, "Expected ';'")?;
        }

        Ok(Statement {
            kind: StatementKind::VarDeclare {
                visibility,
                kind,
                id,
                ty,
                value,
            },
            line: start_line,
            col: start_col,
        })
    }

    fn parse_assign(&mut self) -> Result<Statement<'a>, Error> {
        let start_line = self.peek(0)?.line;
        let start_col = self.peek(0)?.col;

        self.advance()?;

        let var = self.parse_expr()?;

        self.consume(TokenKind::Assign, "Expected '='")?;
        let value = self.parse_expr()?;

        if !matches!(value.kind, ExprKind::Block(_)) {
            self.consume(TokenKind::Semicolon, "Expected ';'")?;
        }

        Ok(Statement {
            kind: StatementKind::VarAssign { var, value },
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
        let ty = self.parse_expr()?;
        self.consume(TokenKind::Assign, "Expected '='")?;
        let value = self.parse_expr()?;

        if !matches!(value.kind, ExprKind::Block(_)) {
            self.consume(TokenKind::Semicolon, "Expected ';'")?;
        }

        Ok(Statement {
            kind: StatementKind::Node {
                visibility,
                id,
                ty,
                value,
            },
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

        let ty = self.parse_expr()?;
        let body = self.parse_expr()?;

        if !matches!(body.kind, ExprKind::Block(_)) {
            self.consume(TokenKind::Semicolon, "Expected ';'")?;
        }

        Ok(Statement {
            kind: StatementKind::Func {
                visibility,
                id,
                params,
                ty,
                body,
            },
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

        Ok(Statement {
            kind: StatementKind::State {
                visibility,
                id,
                fields,
            },
            line: start_line,
            col: start_col,
        })
    }

    fn parse_container(&mut self, visibility: Visibility) -> Result<Statement<'a>, Error> {
        let start_line = self.peek(0)?.line;
        let start_col = self.peek(0)?.col;

        self.advance()?;

        let id = if let TokenKind::Identifier(name) = self.peek(0)?.kind {
            self.advance()?;
            name
        } else {
            return self.error("Expected container name");
        };

        self.consume(TokenKind::LBrace, "Expected '{'")?;
        let types = self.parse_comma_separated(|p| p.parse_expr())?;
        self.consume(TokenKind::RBrace, "Expected '}'")?;

        Ok(Statement {
            kind: StatementKind::Container {
                visibility,
                id,
                types,
            },
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

        Ok(Statement {
            kind: StatementKind::Enum {
                visibility,
                id,
                variants,
            },
            line: start_line,
            col: start_col,
        })
    }

    fn parse_method(&mut self) -> Result<Statement<'a>, Error> {
        let start_line = self.peek(0)?.line;
        let start_col = self.peek(0)?.col;

        self.advance()?;

        let target = self.parse_expr()?;

        let methods_line = self.peek(0)?.line;
        let methods_col = self.peek(0)?.col;
        let methods = self.parse_expr()?;

        if let ExprKind::Block(stmts) = &methods.kind {
            for stmt in stmts {
                if !matches!(stmt.kind, StatementKind::Func { .. }) {
                    return loc_error(
                        methods_line,
                        methods_col,
                        "Method statements must be functions",
                    );
                }
            }
        } else {
            return loc_error(
                methods_line,
                methods_col,
                "Method body must be a block expression",
            );
        }

        Ok(Statement {
            kind: StatementKind::Method { target, methods },
            line: start_line,
            col: start_col,
        })
    }

    fn parse_alias(&mut self) -> Result<Statement<'a>, Error> {
        let start_line = self.peek(0)?.line;
        let start_col = self.peek(0)?.col;

        self.advance()?;

        let TokenKind::Identifier(alias_id) = self.peek(0)?.kind else {
            return self.error("Expected type alias");
        };

        self.advance()?;
        self.consume(TokenKind::Assign, "Expected '='")?;

        let ty = self.parse_expr()?;

        self.consume(TokenKind::Semicolon, "Expected ';'")?;

        Ok(Statement {
            kind: StatementKind::Alias { alias_id, ty },
            line: start_line,
            col: start_col,
        })
    }

    fn parse_statement_expr(&mut self) -> Result<Statement<'a>, Error> {
        let start_line = self.peek(0)?.line;
        let start_col = self.peek(0)?.col;
        let expr = self.parse_expr()?;

        if !self.is_brace_terminated(&expr) {
            self.consume(TokenKind::Semicolon, "Expected ';'")?;
        }

        Ok(Statement {
            kind: StatementKind::Expr(expr),
            line: start_line,
            col: start_col,
        })
    }

    fn parse_visibility(&mut self) -> Result<Statement<'a>, Error> {
        self.advance()?;
        match self.peek(0)?.kind {
            TokenKind::Var => self.parse_var(VarKind::Var, Visibility::Pub),
            TokenKind::NodeStmt => self.parse_node(Visibility::Pub),
            TokenKind::Const => self.parse_var(VarKind::Const, Visibility::Pub),
            TokenKind::FuncStmt => self.parse_func(Visibility::Pub),
            TokenKind::State => self.parse_state(Visibility::Pub),
            TokenKind::Container => self.parse_container(Visibility::Pub),
            TokenKind::Pub => self.error("Repeated visibility modifier"),
            _ => self.error("Invalid visibility modifier"),
        }
    }
}
