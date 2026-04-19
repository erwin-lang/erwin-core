use crate::{
    error::Error,
    lexer::Lexer,
    structure::token::{Token, TokenKind},
};

impl<'a> Lexer<'a> {
    pub(super) fn tokenize_chars(&mut self) -> Result<Token<'a>, Error> {
        match self.peek() {
            Some(c) => match c {
                'a'..='z' | 'A'..='Z' | '_' => self.tokenize_identifier(),
                '0'..='9' => self.tokenize_number(),
                '"' => self.tokenize_string(),
                '#' => self.tokenize_raw_string(),
                '@' => {
                    self.advance();
                    Ok(Token::new(TokenKind::At, self.line, self.column))
                }
                '=' => {
                    self.tokenize_double_symbol(Some(TokenKind::Assign), &[('=', TokenKind::Equal)])
                }
                ':' => self.tokenize_double_symbol(
                    Some(TokenKind::Colon),
                    &[(':', TokenKind::DoubleColon)],
                ),
                '!' => self.tokenize_double_symbol(
                    Some(TokenKind::Not),
                    &[
                        ('=', TokenKind::NotEqual),
                        ('|', TokenKind::Nor),
                        ('&', TokenKind::Nand),
                        ('^', TokenKind::Xnor),
                    ],
                ),
                '<' => self.tokenize_double_symbol(
                    Some(TokenKind::LessThan),
                    &[('=', TokenKind::LessEqual)],
                ),
                '>' => self.tokenize_double_symbol(
                    Some(TokenKind::GreaterThan),
                    &[('=', TokenKind::GreaterEqual)],
                ),
                '|' => self
                    .tokenize_double_symbol(None, &[('|', TokenKind::Or), ('>', TokenKind::RPipe)]),
                '&' => self.tokenize_double_symbol(Some(TokenKind::Amp), &[('&', TokenKind::And)]),
                '^' => self.tokenize_double_symbol(Some(TokenKind::Pow), &[('^', TokenKind::Xor)]),
                '(' => {
                    self.advance();
                    Ok(Token::new(TokenKind::LParen, self.line, self.column))
                }
                ')' => {
                    self.advance();
                    Ok(Token::new(TokenKind::RParen, self.line, self.column))
                }
                '[' => {
                    self.advance();
                    Ok(Token::new(TokenKind::LSquare, self.line, self.column))
                }
                ']' => {
                    self.advance();
                    Ok(Token::new(TokenKind::RSquare, self.line, self.column))
                }
                '{' => {
                    self.advance();
                    Ok(Token::new(TokenKind::LBrace, self.line, self.column))
                }
                '}' => {
                    self.advance();
                    Ok(Token::new(TokenKind::RBrace, self.line, self.column))
                }
                ';' => {
                    self.advance();
                    Ok(Token::new(TokenKind::Semicolon, self.line, self.column))
                }
                ',' => {
                    self.advance();
                    Ok(Token::new(TokenKind::Comma, self.line, self.column))
                }
                '.' => self
                    .tokenize_double_symbol(Some(TokenKind::Dot), &[('.', TokenKind::DoubleDot)]),
                '+' => {
                    self.advance();
                    Ok(Token::new(TokenKind::Plus, self.line, self.column))
                }
                '-' => {
                    self.tokenize_double_symbol(Some(TokenKind::Minus), &[('>', TokenKind::RArrow)])
                }
                '*' => {
                    self.advance();
                    Ok(Token::new(TokenKind::Star, self.line, self.column))
                }
                '/' => {
                    self.advance();
                    Ok(Token::new(TokenKind::Slash, self.line, self.column))
                }
                _ => Err(Error::Custom(format!(
                    "[{}, {}] Unexpected token",
                    self.line, self.column
                ))),
            },
            None => Err(Error::Custom("Unexpected end of file".to_string())),
        }
    }

    pub(super) fn tokenize_identifier(&mut self) -> Result<Token<'a>, Error> {
        let start_index = self.current;
        while let Some(char) = self.peek() {
            if char.is_alphanumeric() || char == '_' {
                self.advance();
            } else {
                break;
            }
        }
        let identifier = &self.code[start_index..self.current];

        let kind = match identifier {
            "var" => TokenKind::Var,
            "mut" => TokenKind::Mut,
            "node" => TokenKind::Node,
            "const" => TokenKind::Const,
            "state" => TokenKind::State,
            "enum" => TokenKind::Enum,
            "method" => TokenKind::Method,
            "func" => TokenKind::Func,
            "return" => TokenKind::Return,
            "yield" => TokenKind::Yield,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "while" => TokenKind::While,
            "continue" => TokenKind::Continue,
            "break" => TokenKind::Break,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "pub" => TokenKind::Pub,
            "import" => TokenKind::Import,
            "Bool" => TokenKind::Bool,
            "Int8" => TokenKind::Int8,
            "Int16" => TokenKind::Int16,
            "Int32" => TokenKind::Int32,
            "Int64" => TokenKind::Int64,
            "Int128" => TokenKind::Int128,
            "UInt8" => TokenKind::UInt8,
            "UInt16" => TokenKind::UInt16,
            "UInt32" => TokenKind::UInt32,
            "UInt64" => TokenKind::UInt64,
            "UInt128" => TokenKind::UInt128,
            "Range8" => TokenKind::IntRange8,
            "Range16" => TokenKind::IntRange16,
            "Range32" => TokenKind::IntRange32,
            "Range64" => TokenKind::IntRange64,
            "Range128" => TokenKind::IntRange128,
            "URange8" => TokenKind::UIntRange8,
            "URange16" => TokenKind::UIntRange16,
            "URange32" => TokenKind::UIntRange32,
            "URange64" => TokenKind::UIntRange64,
            "URange128" => TokenKind::UIntRange128,
            "Float32" => TokenKind::Float32,
            "Float64" => TokenKind::Float64,
            "Str" => TokenKind::String,
            _ => TokenKind::Identifier(identifier),
        };

        Ok(Token::new(kind, self.line, self.column))
    }

    pub(super) fn tokenize_number(&mut self) -> Result<Token<'a>, Error> {
        let start_line = self.line;
        let start_col = self.column;
        let start_index = self.current;
        let mut has_dot = false;

        while let Some(char) = self.peek() {
            if char.is_ascii_digit() {
                self.advance();
            } else if !has_dot && char == '.' {
                if let Some(next) = self.peek_next() {
                    if !next.is_ascii_digit() {
                        break;
                    }
                    has_dot = true;
                    self.advance();
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(Token::new(
            TokenKind::Number(&self.code[start_index..self.current]),
            start_line,
            start_col,
        ))
    }

    pub(super) fn tokenize_string(&mut self) -> Result<Token<'a>, Error> {
        let start_line = self.line;
        let start_col = self.column;

        self.advance();

        let start_index = self.current;

        while let Some(c) = self.peek() {
            match c {
                '"' => break,
                '\\' => {
                    self.advance();
                    if self.peek().is_none() {
                        break;
                    }
                    self.advance();
                }
                _ => self.advance(),
            }
        }

        if self.is_at_end() {
            return Err(Error::Custom(format!(
                "[{}, {}] Unterminated string literal",
                start_line, start_col
            )));
        }

        let value = &self.code[start_index..self.current];
        self.advance();

        Ok(Token::new(
            TokenKind::StringLiteral(value),
            start_line,
            start_col,
        ))
    }

    pub(super) fn tokenize_raw_string(&mut self) -> Result<Token<'a>, Error> {
        let start_line = self.line;
        let start_col = self.column;

        self.advance();

        let start_index = self.current;

        while let Some(c) = self.peek() {
            match c {
                '"' => break,
                _ => self.advance(),
            }
        }

        if self.is_at_end() {
            return Err(Error::Custom(format!(
                "[{}, {}] Unterminated string literal",
                start_line, start_col
            )));
        }

        let value = &self.code[start_index..self.current];
        self.advance();

        Ok(Token::new(
            TokenKind::StringLiteral(value),
            start_line,
            start_col,
        ))
    }

    pub(super) fn tokenize_double_symbol(
        &mut self,
        fallback: Option<TokenKind<'a>>,
        matches: &[(char, TokenKind<'a>)],
    ) -> Result<Token<'a>, Error> {
        self.advance();
        let peek = self.peek();
        let start_line = self.line;
        let start_col = self.column;

        for (c, k) in matches {
            if Some(*c) == peek {
                self.advance();
                return Ok(Token::new(*k, start_line, start_col));
            }
        }

        match fallback {
            Some(f) => Ok(Token::new(f, start_line, start_col)),
            None => Err(Error::Custom(format!(
                "[{}, {}] Unexpected token",
                self.line, self.column
            ))),
        }
    }
}
