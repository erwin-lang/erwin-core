use std::{iter::Peekable, str::Chars};

use crate::{error::Error, parser::token::Token};

pub fn tokenize(code: &str) -> Result<Vec<Token>, Error> {
    let mut chars = code.chars().peekable();
    let mut tokens = Vec::new();

    while let Some(char) = chars.next() {
        match char {
            // Whitespace and newline
            ' ' | '\n' | '\t' | '\r' => {}

            // Identifier and keywords
            'a'..='z' | 'A'..='Z' | '_' => tokens.push(tokenize_identifier(char, &mut chars)?),

            // Number
            '0'..='9' => tokens.push(tokenize_number(char, &mut chars)?),

            // Strings
            '"' => tokens.push(tokenize_string(&mut chars)?),
            '@' => tokens.push(tokenize_raw_string(&mut chars)?),

            // Symbols
            '=' => match chars.peek() {
                Some('=') => push_and_advance(&mut chars, &mut tokens, Token::Equal),
                _ => tokens.push(Token::Assign),
            },
            ':' => match chars.peek() {
                Some('=') => push_and_advance(&mut chars, &mut tokens, Token::NodeAssign),
                _ => tokens.push(Token::Colon),
            },
            '!' => match chars.peek() {
                Some('=') => push_and_advance(&mut chars, &mut tokens, Token::NotEqual),
                Some('|') => push_and_advance(&mut chars, &mut tokens, Token::Nor),
                Some('&') => push_and_advance(&mut chars, &mut tokens, Token::Nand),
                Some('^') => push_and_advance(&mut chars, &mut tokens, Token::Xnor),
                _ => tokens.push(Token::Not),
            },
            '<' => match chars.peek() {
                Some('=') => push_and_advance(&mut chars, &mut tokens, Token::LessEqual),
                Some('|') => push_and_advance(&mut chars, &mut tokens, Token::LPipe),
                Some('-') => push_and_advance(&mut chars, &mut tokens, Token::LArrow),
                _ => tokens.push(Token::LessThan),
            },
            '>' => match chars.peek() {
                Some('=') => push_and_advance(&mut chars, &mut tokens, Token::GreaterEqual),
                _ => tokens.push(Token::GreaterThan),
            },
            '|' => match chars.peek() {
                Some('|') => push_and_advance(&mut chars, &mut tokens, Token::Or),
                Some('>') => push_and_advance(&mut chars, &mut tokens, Token::RPipe),
                _ => return Err(Error::Custom(format!("Unexpected token: {}", char))),
            },
            '&' => match chars.peek() {
                Some('&') => push_and_advance(&mut chars, &mut tokens, Token::And),
                _ => return Err(Error::Custom(format!("Unexpected token: {}", char))),
            },
            '^' => match chars.peek() {
                Some('^') => push_and_advance(&mut chars, &mut tokens, Token::Xor),
                _ => tokens.push(Token::Pow),
            },
            '(' => tokens.push(Token::LParen),
            ')' => tokens.push(Token::RParen),
            '[' => tokens.push(Token::LSquare),
            ']' => tokens.push(Token::RSquare),
            '{' => tokens.push(Token::LBrace),
            '}' => tokens.push(Token::RBrace),
            ';' => tokens.push(Token::Semicolon),
            ',' => tokens.push(Token::Comma),
            '.' => tokens.push(Token::Dot),
            '+' => tokens.push(Token::Plus),
            '-' => match chars.peek() {
                Some('>') => push_and_advance(&mut chars, &mut tokens, Token::RArrow),
                _ => tokens.push(Token::Minus),
            },
            '*' => tokens.push(Token::Star),
            '/' => tokens.push(Token::Slash),

            // Unexpected
            _ => return Err(Error::Custom(format!("Unexpected token: {}", char))),
        }
    }

    tokens.push(Token::EOF);
    Ok(tokens)
}

fn push_and_advance(chars: &mut Peekable<Chars>, tokens: &mut Vec<Token>, token: Token) {
    chars.next();
    tokens.push(token);
}

fn tokenize_identifier(first_char: char, chars: &mut Peekable<Chars>) -> Result<Token, Error> {
    let mut identifier = first_char.to_string();
    while let Some(&char) = chars.peek() {
        if char.is_alphanumeric() || char == '_' {
            identifier.push(char);
            chars.next();
        } else {
            break;
        }
    }

    Ok(match identifier.as_str() {
        "obj" => Token::Obj,
        "func" => Token::Func,
        "return" => Token::Return,
        "for" => Token::For,
        "while" => Token::While,
        "continue" => Token::Continue,
        "break" => Token::Break,
        "if" => Token::If,
        "else" => Token::Else,
        "true" => Token::True,
        "false" => Token::False,
        _ => Token::Identifier(identifier),
    })
}

fn tokenize_number(first_char: char, chars: &mut Peekable<Chars>) -> Result<Token, Error> {
    let mut number = first_char.to_string();
    let mut has_dot = false;
    while let Some(&char) = chars.peek() {
        if char.is_ascii_digit() {
            number.push(char);
            chars.next();
        } else if !has_dot && char == '.' {
            let mut lookahead = chars.clone();
            lookahead.next();
            if let Some(&char_after_dot) = lookahead.peek() {
                if !char_after_dot.is_ascii_digit() {
                    break;
                }
                has_dot = true;
                number.push(char);
                chars.next();
            } else {
                break;
            }
        } else {
            break;
        }
    }

    Ok(Token::Number(number))
}

fn tokenize_string(chars: &mut Peekable<Chars>) -> Result<Token, Error> {
    let mut string = String::new();
    while let Some(char) = chars.next() {
        match char {
            '"' => return Ok(Token::String(string)),
            '\\' => {
                let next = chars
                    .next()
                    .ok_or(Error::Custom("Invalid escape".to_string()))?;
                match next {
                    'n' => string.push('\n'),
                    't' => string.push('\t'),
                    '"' => string.push('\"'),
                    '\\' => string.push('\\'),
                    _ => return Err(Error::Custom("Unknown escape".to_string())),
                }
            }
            _ => string.push(char),
        }
    }

    Err(Error::Custom("Unterminated string".to_string()))
}

fn tokenize_raw_string(chars: &mut Peekable<Chars>) -> Result<Token, Error> {
    let mut string = String::new();
    if chars.next() != Some('"') {
        return Err(Error::Custom("Invalid raw string".to_string()));
    }

    while let Some(char) = chars.next() {
        match char {
            '"' => return Ok(Token::String(string)),
            _ => string.push(char),
        }
    }

    Err(Error::Custom("Unterminated string".to_string()))
}
