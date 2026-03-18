use crate::{
    error::Error,
    parser::Parser,
    syntax::{
        ast::{BinaryOp, Expr, UnaryOp},
        token::TokenKind,
    },
};

impl<'a> Parser<'a> {
    pub(super) fn parse_expr(&mut self) -> Result<Expr<'a>, Error> {
        self.parse_lambda()
    }

    fn parse_lambda(&mut self) -> Result<Expr<'a>, Error> {
        let start_line = self.peek(0)?.line;
        let start_col = self.peek(0)?.column;
        let mut expr = self.parse_pipe()?;

        if matches!(self.peek(0)?.kind, TokenKind::RArrow) {
            self.advance()?;

            let body = self.parse_lambda()?;
            let param_str = match expr {
                Expr::Identifier(name) => name,
                _ => {
                    return self.loc_error(
                        start_line,
                        start_col,
                        "Invalid lambda function parameter",
                    );
                }
            };
            expr = Expr::Lambda {
                param: param_str,
                body: Box::new(body),
            }
        }

        Ok(expr)
    }

    fn parse_pipe(&mut self) -> Result<Expr<'a>, Error> {
        let left = self.parse_or_nor()?;

        match self.peek(0)?.kind {
            TokenKind::LPipe => {
                self.advance()?;
                let right = self.parse_pipe()?;

                Ok(Expr::Binary {
                    left: Box::new(left),
                    op: BinaryOp::LPipe,
                    right: Box::new(right),
                })
            }
            TokenKind::RPipe => {
                let mut expr = left;
                while matches!(self.peek(0)?.kind, TokenKind::RPipe) {
                    self.advance()?;
                    let right = self.parse_or_nor()?;

                    expr = Expr::Binary {
                        left: Box::new(expr),
                        op: BinaryOp::RPipe,
                        right: Box::new(right),
                    };
                }

                Ok(expr)
            }
            _ => Ok(left),
        }
    }

    fn parse_or_nor(&mut self) -> Result<Expr<'a>, Error> {
        let mut left = self.parse_xor_xnor()?;

        while matches!(self.peek(0)?.kind, TokenKind::Or | TokenKind::Nor) {
            let op = match self.peek(0)?.kind {
                TokenKind::Or => BinaryOp::Or,
                TokenKind::Nor => BinaryOp::Nor,
                _ => unreachable!(),
            };

            self.advance()?;
            let right = self.parse_xor_xnor()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_xor_xnor(&mut self) -> Result<Expr<'a>, Error> {
        let mut left = self.parse_and_nand()?;

        while matches!(self.peek(0)?.kind, TokenKind::Xor | TokenKind::Xnor) {
            let op = match self.peek(0)?.kind {
                TokenKind::Xor => BinaryOp::Xor,
                TokenKind::Xnor => BinaryOp::Xnor,
                _ => unreachable!(),
            };

            self.advance()?;
            let right = self.parse_and_nand()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_and_nand(&mut self) -> Result<Expr<'a>, Error> {
        let mut left = self.parse_eq()?;

        while matches!(self.peek(0)?.kind, TokenKind::And | TokenKind::Nand) {
            let op = match self.peek(0)?.kind {
                TokenKind::And => BinaryOp::And,
                TokenKind::Nand => BinaryOp::Nand,
                _ => unreachable!(),
            };

            self.advance()?;
            let right = self.parse_eq()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_eq(&mut self) -> Result<Expr<'a>, Error> {
        let mut left = self.parse_cmp()?;

        while matches!(self.peek(0)?.kind, TokenKind::Equal | TokenKind::NotEqual) {
            let op = match self.peek(0)?.kind {
                TokenKind::Equal => BinaryOp::Equal,
                TokenKind::NotEqual => BinaryOp::NotEqual,
                _ => unreachable!(),
            };

            self.advance()?;
            let right = self.parse_cmp()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_cmp(&mut self) -> Result<Expr<'a>, Error> {
        let mut left = self.parse_add_sub()?;

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
            let right = self.parse_add_sub()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_add_sub(&mut self) -> Result<Expr<'a>, Error> {
        let mut left = self.parse_mult_div()?;

        while matches!(self.peek(0)?.kind, TokenKind::Plus | TokenKind::Minus) {
            let op = match self.peek(0)?.kind {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Sub,
                _ => unreachable!(),
            };

            self.advance()?;
            let right = self.parse_mult_div()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_mult_div(&mut self) -> Result<Expr<'a>, Error> {
        let mut left = self.parse_pow()?;

        while matches!(self.peek(0)?.kind, TokenKind::Star | TokenKind::Slash) {
            let op = match self.peek(0)?.kind {
                TokenKind::Star => BinaryOp::Mult,
                TokenKind::Slash => BinaryOp::Div,
                _ => unreachable!(),
            };

            self.advance()?;
            let right = self.parse_pow()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_pow(&mut self) -> Result<Expr<'a>, Error> {
        let left = self.parse_unary()?;

        if matches!(self.peek(0)?.kind, TokenKind::Pow) {
            self.advance()?;
            let right = self.parse_pow()?;

            return Ok(Expr::Binary {
                left: Box::new(left),
                op: BinaryOp::Pow,
                right: Box::new(right),
            });
        };

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr<'a>, Error> {
        if matches!(self.peek(0)?.kind, TokenKind::Not | TokenKind::Minus) {
            let op = match self.peek(0)?.kind {
                TokenKind::Not => UnaryOp::Not,
                TokenKind::Minus => UnaryOp::Minus,
                _ => unreachable!(),
            };

            self.advance()?;
            let right = self.parse_unary()?;
            return Ok(Expr::Unary {
                op,
                right: Box::new(right),
            });
        }

        self.parse_call()
    }

    fn parse_call(&mut self) -> Result<Expr<'a>, Error> {
        let mut base = self.parse_primary()?;

        while matches!(self.peek(0)?.kind, TokenKind::LParen) {
            self.advance()?;

            if matches!(self.peek(0)?.kind, TokenKind::RParen) {
                self.advance()?;
                base = Expr::Call {
                    base: Box::new(base),
                    args: None,
                };
                continue;
            }

            let mut args = Vec::new();
            args.push(self.parse_expr()?);

            loop {
                match self.peek(0)?.kind {
                    TokenKind::Comma => {
                        self.advance()?;
                        args.push(self.parse_expr()?);
                    }
                    TokenKind::RParen => {
                        self.advance()?;
                        break;
                    }
                    _ => return self.error("Expected ',' or ')'"),
                }
            }

            base = Expr::Call {
                base: Box::new(base),
                args: Some(args),
            }
        }

        Ok(base)
    }

    fn parse_primary(&mut self) -> Result<Expr<'a>, Error> {
        match self.peek(0)?.kind {
            TokenKind::Number(num) => {
                self.advance()?;
                Ok(Expr::Number(num))
            }
            TokenKind::StringLiteral(s) => {
                self.advance()?;
                Ok(Expr::String(s))
            }
            TokenKind::True => {
                self.advance()?;
                Ok(Expr::Bool(true))
            }
            TokenKind::False => {
                self.advance()?;
                Ok(Expr::Bool(false))
            }
            TokenKind::Identifier(id) => {
                if let Ok(path) = self.parse_path() {
                    Ok(Expr::Path(path))
                } else {
                    self.advance()?;
                    Ok(Expr::Identifier(id))
                }
            }
            TokenKind::LParen => {
                self.advance()?;
                let expr = self.parse_expr()?;

                if matches!(self.peek(0)?.kind, TokenKind::Comma) {
                    let mut items = vec![expr];

                    while matches!(self.peek(0)?.kind, TokenKind::Comma) {
                        self.advance()?;
                        items.push(self.parse_expr()?);
                    }
                    if !matches!(self.peek(0)?.kind, TokenKind::RParen) {
                        return self.error("Unterminated tuple, expected ')'");
                    }

                    self.advance()?;
                    Ok(Expr::Tuple(items))
                } else {
                    if !matches!(self.peek(0)?.kind, TokenKind::RParen) {
                        return self.error("Unterminated group, expected ')'");
                    }

                    self.advance()?;
                    Ok(expr)
                }
            }
            TokenKind::LSquare => {
                self.advance()?;
                let mut items = Vec::new();
                items.push(self.parse_expr()?);

                while matches!(self.peek(0)?.kind, TokenKind::Comma) {
                    self.advance()?;
                    items.push(self.parse_expr()?);
                }
                if !matches!(self.peek(0)?.kind, TokenKind::RSquare) {
                    return self.error("Unterminated array, expected ']'");
                }

                self.advance()?;
                Ok(Expr::Array(items))
            }
            _ => {
                return self.error("Invalid expression");
            }
        }
    }
}
