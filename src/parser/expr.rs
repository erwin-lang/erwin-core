use crate::{
    error::Error,
    parser::Parser,
    structure::{
        ast::{BinaryOp, Expr, ExprKind, UnaryOp},
        token::TokenKind,
    },
};

impl<'a> Parser<'a> {
    pub(super) fn parse_expr(&mut self) -> Result<Expr<'a>, Error> {
        self.parse_lambda()
    }

    fn parse_lambda(&mut self) -> Result<Expr<'a>, Error> {
        let start_line = self.peek(0)?.line;
        let start_col = self.peek(0)?.col;

        if !matches!(self.peek(0)?.kind, TokenKind::LParen) {
            return self.parse_control();
        }

        let mut i = 1;
        let mut depth = 1;

        while depth > 0 {
            match self.peek(i)?.kind {
                TokenKind::LParen => depth += 1,
                TokenKind::RParen => depth -= 1,
                TokenKind::Eof => break,
                _ => {}
            }

            i += 1;
        }

        if matches!(self.peek(i)?.kind, TokenKind::RArrow) {
            self.consume(TokenKind::LParen, "Expected '('")?;
            let params = self.parse_comma_separated(|p| p.parse_param())?;
            self.consume(TokenKind::RParen, "Expected ')'")?;
            self.consume(TokenKind::RArrow, "Expected '->'")?;
            let body = self.parse_lambda()?;

            return Ok(Expr {
                kind: ExprKind::Lambda {
                    params,
                    body: Box::new(body),
                },
                line: start_line,
                col: start_col,
            });
        }

        self.parse_control()
    }

    fn parse_control(&mut self) -> Result<Expr<'a>, Error> {
        let start_line = self.peek(0)?.line;
        let start_col = self.peek(0)?.col;

        match self.peek(0)?.kind {
            TokenKind::If => {
                self.advance()?;
                let condition = Box::new(self.parse_expr()?);

                self.consume(TokenKind::Do, "Expected 'do'")?;
                let do_body = Box::new(self.parse_expr()?);

                let mut else_body = None;
                if matches!(self.peek(0)?.kind, TokenKind::Else) {
                    self.advance()?;
                    else_body = Some(Box::new(self.parse_expr()?));
                }

                let kind = ExprKind::If {
                    condition,
                    do_body,
                    else_body,
                };

                Ok(Expr {
                    kind,
                    line: start_line,
                    col: start_col,
                })
            }
            TokenKind::For => {
                self.advance()?;
                let elem = if let ExprKind::Identifier(id) = self.parse_expr()?.kind {
                    id
                } else {
                    return self.error("Expected identifier");
                };

                self.consume(TokenKind::In, "Expected 'in'")?;
                let iter = Box::new(self.parse_expr()?);

                self.consume(TokenKind::Do, "Expected 'do'")?;
                let do_body = Box::new(self.parse_expr()?);

                let mut else_body = None;
                if matches!(self.peek(0)?.kind, TokenKind::Else) {
                    self.advance()?;
                    else_body = Some(Box::new(self.parse_expr()?));
                }

                let kind = ExprKind::For {
                    elem,
                    iter,
                    do_body,
                    else_body,
                };

                Ok(Expr {
                    kind,
                    line: start_line,
                    col: start_col,
                })
            }
            TokenKind::While => {
                self.advance()?;
                let condition = Box::new(self.parse_expr()?);

                self.consume(TokenKind::Do, "Expected 'do'")?;
                let do_body = Box::new(self.parse_expr()?);

                let mut else_body = None;
                if matches!(self.peek(0)?.kind, TokenKind::Else) {
                    self.advance()?;
                    else_body = Some(Box::new(self.parse_expr()?));
                }

                let kind = ExprKind::While {
                    condition,
                    do_body,
                    else_body,
                };

                Ok(Expr {
                    kind,
                    line: start_line,
                    col: start_col,
                })
            }
            _ => self.parse_pipe(),
        }
    }

    fn parse_pipe(&mut self) -> Result<Expr<'a>, Error> {
        let start_line = self.peek(0)?.line;
        let start_col = self.peek(0)?.col;

        let mut left = self.parse_or_nor()?;

        while matches!(self.peek(0)?.kind, TokenKind::RPipe) {
            self.advance()?;
            let right = self.parse_or_nor()?;

            let kind = ExprKind::Binary {
                left: Box::new(left),
                op: BinaryOp::RPipe,
                right: Box::new(right),
            };

            left = Expr {
                kind,
                line: start_line,
                col: start_col,
            };
        }

        Ok(left)
    }

    fn parse_or_nor(&mut self) -> Result<Expr<'a>, Error> {
        let start_line = self.peek(0)?.line;
        let start_col = self.peek(0)?.col;

        let mut left = self.parse_xor_xnor()?;

        while matches!(self.peek(0)?.kind, TokenKind::Or | TokenKind::Nor) {
            let op = match self.peek(0)?.kind {
                TokenKind::Or => BinaryOp::Or,
                TokenKind::Nor => BinaryOp::Nor,
                _ => unreachable!(),
            };

            self.advance()?;
            let right = self.parse_xor_xnor()?;
            let kind = ExprKind::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };

            left = Expr {
                kind,
                line: start_line,
                col: start_col,
            };
        }

        Ok(left)
    }

    fn parse_xor_xnor(&mut self) -> Result<Expr<'a>, Error> {
        let start_line = self.peek(0)?.line;
        let start_col = self.peek(0)?.col;

        let mut left = self.parse_and_nand()?;

        while matches!(self.peek(0)?.kind, TokenKind::Xor | TokenKind::Xnor) {
            let op = match self.peek(0)?.kind {
                TokenKind::Xor => BinaryOp::Xor,
                TokenKind::Xnor => BinaryOp::Xnor,
                _ => unreachable!(),
            };

            self.advance()?;
            let right = self.parse_and_nand()?;
            let kind = ExprKind::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };

            left = Expr {
                kind,
                line: start_line,
                col: start_col,
            };
        }

        Ok(left)
    }

    fn parse_and_nand(&mut self) -> Result<Expr<'a>, Error> {
        let start_line = self.peek(0)?.line;
        let start_col = self.peek(0)?.col;

        let mut left = self.parse_eq()?;

        while matches!(self.peek(0)?.kind, TokenKind::And | TokenKind::Nand) {
            let op = match self.peek(0)?.kind {
                TokenKind::And => BinaryOp::And,
                TokenKind::Nand => BinaryOp::Nand,
                _ => unreachable!(),
            };

            self.advance()?;
            let right = self.parse_eq()?;
            let kind = ExprKind::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };

            left = Expr {
                kind,
                line: start_line,
                col: start_col,
            };
        }

        Ok(left)
    }

    fn parse_eq(&mut self) -> Result<Expr<'a>, Error> {
        let start_line = self.peek(0)?.line;
        let start_col = self.peek(0)?.col;

        let mut left = self.parse_cmp()?;

        while matches!(self.peek(0)?.kind, TokenKind::Equal | TokenKind::NotEqual) {
            let op = match self.peek(0)?.kind {
                TokenKind::Equal => BinaryOp::Equal,
                TokenKind::NotEqual => BinaryOp::NotEqual,
                _ => unreachable!(),
            };

            self.advance()?;
            let right = self.parse_cmp()?;
            let kind = ExprKind::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };

            left = Expr {
                kind,
                line: start_line,
                col: start_col,
            };
        }

        Ok(left)
    }

    fn parse_cmp(&mut self) -> Result<Expr<'a>, Error> {
        let start_line = self.peek(0)?.line;
        let start_col = self.peek(0)?.col;

        let mut left = self.parse_range()?;

        while matches!(
            self.peek(0)?.kind,
            TokenKind::LessThan
                | TokenKind::GreaterThan
                | TokenKind::LessEqual
                | TokenKind::GreaterEqual
        ) {
            let op = match self.peek(0)?.kind {
                TokenKind::LessThan => BinaryOp::LessThan,
                TokenKind::GreaterThan => BinaryOp::GreaterThan,
                TokenKind::LessEqual => BinaryOp::LessEqual,
                TokenKind::GreaterEqual => BinaryOp::GreaterEqual,
                _ => unreachable!(),
            };

            self.advance()?;
            let right = self.parse_range()?;
            let kind = ExprKind::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };

            left = Expr {
                kind,
                line: start_line,
                col: start_col,
            };
        }

        Ok(left)
    }

    fn parse_range(&mut self) -> Result<Expr<'a>, Error> {
        let start_line = self.peek(0)?.line;
        let start_col = self.peek(0)?.col;

        let left = self.parse_add_sub()?;

        if matches!(self.peek(0)?.kind, TokenKind::DoubleDot) {
            self.advance()?;
            let right = self.parse_add_sub()?;

            return Ok(Expr {
                kind: ExprKind::Binary {
                    left: Box::new(left),
                    op: BinaryOp::Range,
                    right: Box::new(right),
                },
                line: start_line,
                col: start_col,
            });
        }

        Ok(left)
    }

    fn parse_add_sub(&mut self) -> Result<Expr<'a>, Error> {
        let start_line = self.peek(0)?.line;
        let start_col = self.peek(0)?.col;

        let mut left = self.parse_mult_div()?;

        while matches!(self.peek(0)?.kind, TokenKind::Plus | TokenKind::Minus) {
            let op = match self.peek(0)?.kind {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Sub,
                _ => unreachable!(),
            };

            self.advance()?;
            let right = self.parse_mult_div()?;
            let kind = ExprKind::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };

            left = Expr {
                kind,
                line: start_line,
                col: start_col,
            };
        }

        Ok(left)
    }

    fn parse_mult_div(&mut self) -> Result<Expr<'a>, Error> {
        let start_line = self.peek(0)?.line;
        let start_col = self.peek(0)?.col;

        let mut left = self.parse_pow()?;

        while matches!(self.peek(0)?.kind, TokenKind::Star | TokenKind::Slash) {
            let op = match self.peek(0)?.kind {
                TokenKind::Star => BinaryOp::Mult,
                TokenKind::Slash => BinaryOp::Div,
                _ => unreachable!(),
            };

            self.advance()?;
            let right = self.parse_pow()?;
            let kind = ExprKind::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };

            left = Expr {
                kind,
                line: start_line,
                col: start_col,
            };
        }

        Ok(left)
    }

    fn parse_pow(&mut self) -> Result<Expr<'a>, Error> {
        let start_line = self.peek(0)?.line;
        let start_col = self.peek(0)?.col;

        let left = self.parse_unary()?;

        if matches!(self.peek(0)?.kind, TokenKind::Pow) {
            self.advance()?;
            let right = self.parse_pow()?;

            let kind = ExprKind::Binary {
                left: Box::new(left),
                op: BinaryOp::Pow,
                right: Box::new(right),
            };

            return Ok(Expr {
                kind,
                line: start_line,
                col: start_col,
            });
        };

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr<'a>, Error> {
        let start_line = self.peek(0)?.line;
        let start_col = self.peek(0)?.col;

        if matches!(
            self.peek(0)?.kind,
            TokenKind::Not | TokenKind::Minus | TokenKind::Amp | TokenKind::Star
        ) {
            let op = match self.peek(0)?.kind {
                TokenKind::Not => UnaryOp::Not,
                TokenKind::Minus => UnaryOp::Minus,
                TokenKind::Amp => UnaryOp::Ref,
                TokenKind::Star => UnaryOp::Deref,
                _ => unreachable!(),
            };

            self.advance()?;
            let right = self.parse_unary()?;
            let kind = ExprKind::Unary {
                op,
                right: Box::new(right),
            };

            return Ok(Expr {
                kind,
                line: start_line,
                col: start_col,
            });
        }

        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expr<'a>, Error> {
        let start_line = self.peek(0)?.line;
        let start_col = self.peek(0)?.col;

        let mut base = self.parse_primary()?;

        loop {
            match self.peek(0)?.kind {
                TokenKind::LParen => {
                    self.advance()?;

                    let args = self.parse_comma_separated(|parser| parser.parse_expr())?;
                    self.consume(TokenKind::RParen, "Expected ')'")?;

                    let kind = ExprKind::Call {
                        base: Box::new(base),
                        args,
                    };

                    base = Expr {
                        kind,
                        line: start_line,
                        col: start_col,
                    };
                }
                TokenKind::Dot => {
                    self.advance()?;

                    let member = if let TokenKind::Identifier(name) = self.peek(0)?.kind {
                        self.advance()?;
                        name
                    } else {
                        return self.error("Expected member name after '.'");
                    };

                    let kind = ExprKind::MemberAccess {
                        target: Box::new(base),
                        member,
                    };

                    base = Expr {
                        kind,
                        line: start_line,
                        col: start_col,
                    };
                }
                TokenKind::DoubleColon => {
                    self.advance()?;

                    let member = if let TokenKind::Identifier(name) = self.peek(0)?.kind {
                        self.advance()?;
                        name
                    } else {
                        return self.error("Expected static entry name after '::'");
                    };

                    let kind = ExprKind::StaticAccess {
                        target: Box::new(base),
                        member,
                    };

                    base = Expr {
                        kind,
                        line: start_line,
                        col: start_col,
                    };
                }
                _ => break,
            }
        }

        Ok(base)
    }

    fn parse_primary(&mut self) -> Result<Expr<'a>, Error> {
        let start_line = self.peek(0)?.line;
        let start_col = self.peek(0)?.col;

        match self.peek(0)?.kind {
            TokenKind::Number(num) => {
                self.advance()?;
                let kind = ExprKind::Number(num);

                Ok(Expr {
                    kind,
                    line: start_line,
                    col: start_col,
                })
            }
            TokenKind::StringLiteral(s) => {
                self.advance()?;
                let kind = ExprKind::String(s);

                Ok(Expr {
                    kind,
                    line: start_line,
                    col: start_col,
                })
            }
            TokenKind::True => {
                self.advance()?;
                let kind = ExprKind::Bool(true);

                Ok(Expr {
                    kind,
                    line: start_line,
                    col: start_col,
                })
            }
            TokenKind::False => {
                self.advance()?;
                let kind = ExprKind::Bool(false);

                Ok(Expr {
                    kind,
                    line: start_line,
                    col: start_col,
                })
            }
            TokenKind::Identifier(id) => match self.peek(1)?.kind {
                TokenKind::LBrace => {
                    self.advance()?;
                    self.advance()?;
                    let fields = self.parse_comma_separated(|p| p.parse_instance_field())?;
                    self.consume(TokenKind::RBrace, "Expected '}'")?;
                    let kind = ExprKind::StateInstance { id, fields };

                    Ok(Expr {
                        kind,
                        line: start_line,
                        col: start_col,
                    })
                }
                _ => {
                    self.advance()?;
                    let kind = ExprKind::Identifier(id);

                    Ok(Expr {
                        kind,
                        line: start_line,
                        col: start_col,
                    })
                }
            },
            TokenKind::LParen => {
                self.advance()?;

                let first_expr = self.parse_expr()?;

                if matches!(self.peek(0)?.kind, TokenKind::Comma) {
                    self.advance()?;

                    let mut items = vec![first_expr];
                    items.extend(self.parse_comma_separated(|p| p.parse_expr())?);
                    self.consume(TokenKind::RParen, "Expected ')'")?;
                    let kind = ExprKind::Tuple(items);

                    Ok(Expr {
                        kind,
                        line: start_line,
                        col: start_col,
                    })
                } else {
                    self.consume(TokenKind::RParen, "Expected ')'")?;
                    Ok(first_expr)
                }
            }
            TokenKind::LSquare => {
                self.advance()?;

                let items = self.parse_comma_separated(|p| p.parse_expr())?;
                self.consume(TokenKind::RSquare, "Expected ']'")?;
                let kind = ExprKind::Array(items);

                Ok(Expr {
                    kind,
                    line: start_line,
                    col: start_col,
                })
            }
            TokenKind::LBrace => {
                let brace_line = self.peek(0)?.line;
                let brace_col = self.peek(0)?.col;
                let mut stmts = Vec::new();
                self.advance()?;

                while !matches!(self.peek(0)?.kind, TokenKind::RBrace | TokenKind::Eof) {
                    stmts.push(self.parse_statement()?);
                }

                if matches!(self.peek(0)?.kind, TokenKind::Eof) {
                    return self.loc_error(brace_line, brace_col, "Unterminated block");
                }

                self.advance()?;

                let kind = ExprKind::Block(stmts);

                Ok(Expr {
                    kind,
                    line: start_line,
                    col: start_col,
                })
            }
            TokenKind::Return => {
                self.advance()?;

                let expr = self.parse_expr()?;
                let kind = ExprKind::Return(Box::new(expr));

                Ok(Expr {
                    kind,
                    line: start_line,
                    col: start_col,
                })
            }
            TokenKind::Yield => {
                self.advance()?;

                let expr = self.parse_expr()?;
                let kind = ExprKind::Yield(Box::new(expr));

                Ok(Expr {
                    kind,
                    line: start_line,
                    col: start_col,
                })
            }
            TokenKind::Break => {
                self.advance()?;
                let kind = ExprKind::Break;

                Ok(Expr {
                    kind,
                    line: start_line,
                    col: start_col,
                })
            }
            TokenKind::Continue => {
                self.advance()?;
                let kind = ExprKind::Continue;

                Ok(Expr {
                    kind,
                    line: start_line,
                    col: start_col,
                })
            }
            _ => self.error("Invalid expression"),
        }
    }
}
