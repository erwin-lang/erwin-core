pub(super) mod chars;

use crate::{
    error::Error,
    structure::token::{Token, TokenKind},
};

pub(crate) struct Lexer<'a> {
    pub(super) code: &'a str,
    pub(super) current: usize,
    pub(super) line: usize,
    pub(super) column: usize,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(code: &'a str) -> Self {
        Self {
            code,
            current: 0,
            line: 1,
            column: 1,
        }
    }

    pub(crate) fn tokenize(mut self) -> Result<Vec<Token<'a>>, Error> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            while let Some(c) = self.peek() {
                if c.is_whitespace() {
                    self.advance();
                } else {
                    break;
                }
            }

            if self.is_at_end() {
                break;
            }

            tokens.push(self.tokenize_chars()?);
        }

        tokens.push(Token::new(TokenKind::Eof, self.line, self.column));
        Ok(tokens)
    }

    pub(super) fn is_at_end(&self) -> bool {
        self.current >= self.code.len()
    }

    pub(super) fn advance(&mut self) {
        if let Some(c) = self.peek() {
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.current += c.len_utf8();
        }
    }

    pub(super) fn peek(&self) -> Option<char> {
        self.code[self.current..].chars().next()
    }

    pub(super) fn peek_next(&self) -> Option<char> {
        let mut chars = self.code[self.current..].chars();
        chars.next();
        chars.next()
    }
}
