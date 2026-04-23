pub(super) mod expr;
pub(super) mod statement;

use crate::{
    error::Error,
    structure::{
        ast::{Expr, ExprKind, Field, InstanceField, Param, Statement, Variant, Visibility},
        token::{Token, TokenKind},
    },
};

pub(crate) struct Parser<'a> {
    pub(super) tokens: Vec<Token<'a>>,
    pub(super) current: usize,
}

impl<'a> Parser<'a> {
    pub(crate) fn new(tokens: Vec<Token<'a>>) -> Self {
        Self { tokens, current: 0 }
    }

    pub(crate) fn parse(mut self) -> Result<Vec<Statement<'a>>, Error> {
        let mut program = Vec::new();

        while !matches!(self.peek(0)?.kind, TokenKind::Eof) {
            program.push(self.parse_statement()?);
        }

        Ok(program)
    }

    pub(super) fn parse_comma_separated<T, F>(&mut self, mut parse_item: F) -> Result<Vec<T>, Error>
    where
        F: FnMut(&mut Self) -> Result<T, Error>,
    {
        let mut items = Vec::new();

        if matches!(
            self.peek(0)?.kind,
            TokenKind::RParen | TokenKind::RSquare | TokenKind::RBrace
        ) {
            return Ok(items);
        }

        loop {
            items.push(parse_item(self)?);

            if matches!(self.peek(0)?.kind, TokenKind::Comma) {
                self.advance()?;

                if matches!(
                    self.peek(0)?.kind,
                    TokenKind::RParen | TokenKind::RSquare | TokenKind::RBrace
                ) {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(items)
    }

    pub(super) fn parse_param(&mut self) -> Result<Param<'a>, Error> {
        let TokenKind::Identifier(id) = self.peek(0)?.kind else {
            return self.error("Expected parameter identifier");
        };

        let start_line = self.peek(0)?.line;
        let start_col = self.peek(0)?.col;
        self.advance()?;

        if id == "self" {
            return Ok(Param {
                id,
                ty: Expr {
                    kind: ExprKind::Identifier("Self"),
                    line: start_line,
                    col: start_col,
                },
            });
        }

        self.consume(TokenKind::Colon, "Expected ':'")?;
        let ty = self.parse_expr()?;

        Ok(Param { id, ty })
    }

    pub(super) fn parse_field(&mut self) -> Result<Field<'a>, Error> {
        let visibility = if matches!(self.peek(0)?.kind, TokenKind::Pub) {
            self.advance()?;
            Visibility::Pub
        } else {
            Visibility::Priv
        };

        let id = match self.peek(0)?.kind {
            TokenKind::Identifier(id) => {
                self.advance()?;
                id
            }
            _ => return self.error("Expected field identifier"),
        };

        self.consume(TokenKind::Colon, "Expected ':'")?;
        let ty = self.parse_expr()?;

        Ok(Field { visibility, id, ty })
    }

    pub(super) fn parse_instance_field(&mut self) -> Result<InstanceField<'a>, Error> {
        let id = match self.peek(0)?.kind {
            TokenKind::Identifier(id) => {
                self.advance()?;
                id
            }
            _ => return self.error("Expected field identifier"),
        };

        self.consume(TokenKind::Colon, "Expected ':'")?;
        let value = self.parse_expr()?;

        Ok(InstanceField { id, value })
    }

    pub(super) fn parse_variant(&mut self) -> Result<Variant<'a>, Error> {
        let id = match self.peek(0)?.kind {
            TokenKind::Identifier(id) => {
                self.advance()?;
                id
            }
            _ => return self.error("Expected variant identifier"),
        };

        let mut data = Vec::new();
        if matches!(self.peek(0)?.kind, TokenKind::LParen) {
            self.advance()?;
            data = self.parse_comma_separated(|p| p.parse_expr())?;
            self.consume(TokenKind::RParen, "Expected ')'")?;
        }

        Ok(Variant { id, data })
    }

    pub(super) fn is_brace_terminated(&self, expr: &Expr<'a>) -> bool {
        match &expr.kind {
            ExprKind::If {
                condition: _,
                do_body,
                else_body,
            }
            | ExprKind::While {
                condition: _,
                do_body,
                else_body,
            }
            | ExprKind::For {
                elem: _,
                iter: _,
                do_body,
                else_body,
            } => {
                if let Some(else_branch) = else_body {
                    self.is_brace_terminated(else_branch)
                } else {
                    self.is_brace_terminated(do_body)
                }
            }
            ExprKind::Block(_) => true,
            _ => false,
        }
    }

    pub(super) fn peek(&self, offset: isize) -> Result<&Token<'a>, Error> {
        let index = self.current as isize + offset;

        if index < 0 {
            return Err(Error::Custom("Parser looked before start".to_string()));
        }

        if index as usize >= self.tokens.len() {
            return Ok(&self.tokens[self.tokens.len() - 1]);
        }

        Ok(&self.tokens[index as usize])
    }

    pub(super) fn advance(&mut self) -> Result<(), Error> {
        if self.current < self.tokens.len() - 1 {
            self.current += 1;
        }
        Ok(())
    }

    pub(super) fn consume(&mut self, _token: TokenKind, err: &str) -> Result<(), Error> {
        if !matches!(self.peek(0)?.kind, _token) {
            return self.error(err);
        }

        self.advance()
    }

    pub(super) fn error<T>(&self, msg: &str) -> Result<T, Error> {
        let token = self.peek(0)?;
        self.loc_error(token.line, token.col, msg)
    }

    pub(super) fn loc_error<T>(&self, line: usize, col: usize, msg: &str) -> Result<T, Error> {
        Err(Error::Custom(format!("[{}, {}] {}", line, col, msg)))
    }
}
