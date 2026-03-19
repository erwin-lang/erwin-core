use crate::error::Error;
use crate::syntax::ast::{Param, Statement};
use crate::syntax::token::{Token, TokenKind};

pub(crate) mod expr;
pub(crate) mod statement;
pub(crate) mod types;

pub(crate) struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    current: usize,
}

impl<'a> Parser<'a> {
    pub(crate) fn new(tokens: Vec<Token<'a>>) -> Self {
        Self { tokens, current: 0 }
    }

    pub(crate) fn parse(&mut self) -> Result<Vec<Statement<'a>>, Error> {
        let mut program = Vec::new();

        while !matches!(self.peek(0)?.kind, TokenKind::EOF) {
            program.push(self.parse_statement()?);
        }

        Ok(program)
    }

    pub(super) fn parse_path(&mut self) -> Result<Vec<&'a str>, Error> {
        let mut path = Vec::new();

        loop {
            if let TokenKind::Identifier(element) = self.peek(0)?.kind {
                path.push(element);
                self.advance()?;
            } else {
                return self.error("Expected identifier in path");
            }

            if matches!(self.peek(0)?.kind, TokenKind::Dot)
                && matches!(self.peek(1)?.kind, TokenKind::Identifier(_))
            {
                self.advance()?;
            } else {
                break;
            }
        }

        Ok(path)
    }

    pub(super) fn parse_block(&mut self) -> Result<Vec<Statement<'a>>, Error> {
        let brace_line = self.peek(0)?.line;
        let brace_col = self.peek(0)?.column;
        let mut stmts = Vec::new();
        self.advance()?;

        while !matches!(self.peek(0)?.kind, TokenKind::EOF) {
            if matches!(self.peek(0)?.kind, TokenKind::RBrace) {
                self.advance()?;
                break;
            }
            stmts.push(self.parse_statement()?);
        }

        if matches!(self.peek(0)?.kind, TokenKind::EOF) {
            return self.loc_error(brace_line, brace_col, "Unterminated block");
        }

        Ok(stmts)
    }

    pub(super) fn parse_comma_separated<T, F>(&mut self, mut parse_item: F) -> Result<Vec<T>, Error>
    where
        F: FnMut(&mut Self) -> Result<T, Error>,
    {
        let mut items = Vec::new();

        if matches!(self.peek(0)?.kind, TokenKind::RParen | TokenKind::RSquare) {
            return Ok(items);
        }

        loop {
            items.push(parse_item(self)?);

            if matches!(self.peek(0)?.kind, TokenKind::Comma) {
                self.advance()?;

                if matches!(self.peek(0)?.kind, TokenKind::RParen | TokenKind::RSquare) {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(items)
    }

    pub(super) fn parse_param(&mut self) -> Result<Param<'a>, Error> {
        let id = match self.peek(0)?.kind {
            TokenKind::Identifier(id) => {
                self.advance()?;
                id
            }
            _ => return self.error("Expected parameter identifier"),
        };

        if !matches!(self.peek(0)?.kind, TokenKind::Colon) {
            return self.error("Expected ':'");
        }

        self.advance()?;

        let ty = self.parse_type()?;

        Ok(Param { identifier: id, ty })
    }

    pub(super) fn peek(&self, offset: isize) -> Result<&Token<'a>, Error> {
        let index = self.current as isize + offset;

        if index < 0 || index as usize >= self.tokens.len() {
            return Err(Error::Custom(
                "Parser is already at the last token".to_string(),
            ));
        }

        Ok(&self.tokens[index as usize])
    }

    pub(super) fn advance(&mut self) -> Result<(), Error> {
        if self.current >= self.tokens.len() {
            return Err(Error::Custom(
                "Parser is already at the last token".to_string(),
            ));
        }
        self.current += 1;
        Ok(())
    }

    pub(super) fn error<T>(&self, msg: &str) -> Result<T, Error> {
        let token = self.peek(0)?;
        self.loc_error(token.line, token.column, msg)
    }

    pub(super) fn loc_error<T>(&self, line: usize, col: usize, msg: &str) -> Result<T, Error> {
        Err(Error::Custom(format!("[{}, {}] {}", line, col, msg)))
    }
}
