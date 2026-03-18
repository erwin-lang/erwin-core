use crate::{
    error::Error,
    parser::Parser,
    syntax::{
        ast::{FloatSize, IntSize, Sign, Type},
        token::TokenKind,
    },
};

impl<'a> Parser<'a> {
    pub(super) fn parse_type(&mut self) -> Result<Type<'a>, Error> {
        let mut base_type = match self.peek(0)?.kind {
            TokenKind::Bool => Type::Bool,
            TokenKind::Int8 => Type::Integer {
                size: IntSize::B8,
                sign: Sign::Signed,
            },
            TokenKind::Int16 => Type::Integer {
                size: IntSize::B16,
                sign: Sign::Signed,
            },
            TokenKind::Int32 => Type::Integer {
                size: IntSize::B32,
                sign: Sign::Signed,
            },
            TokenKind::Int64 => Type::Integer {
                size: IntSize::B64,
                sign: Sign::Signed,
            },
            TokenKind::Uint8 => Type::Integer {
                size: IntSize::B8,
                sign: Sign::Unsigned,
            },
            TokenKind::Uint16 => Type::Integer {
                size: IntSize::B16,
                sign: Sign::Unsigned,
            },
            TokenKind::Uint32 => Type::Integer {
                size: IntSize::B32,
                sign: Sign::Unsigned,
            },
            TokenKind::Uint64 => Type::Integer {
                size: IntSize::B64,
                sign: Sign::Unsigned,
            },
            TokenKind::Float32 => Type::Float {
                size: FloatSize::B32,
            },
            TokenKind::Float64 => Type::Float {
                size: FloatSize::B64,
            },
            TokenKind::String => Type::String,
            TokenKind::LParen => self.parse_tuple_type()?,
            TokenKind::LSquare => self.parse_array_type()?,
            TokenKind::Byte => Type::Byte,
            TokenKind::Identifier(ty) => Type::Custom(ty),
            _ => return self.error("Invalid type"),
        };
        self.advance()?;

        while matches!(self.peek(0)?.kind, TokenKind::Star) {
            self.advance()?;
            base_type = Type::Pointer(Box::new(base_type));
        }
        Ok(base_type)
    }

    fn parse_tuple_type(&mut self) -> Result<Type<'a>, Error> {
        let mut types = Vec::new();
        let paren_line = self.peek(0)?.line;
        let paren_col = self.peek(0)?.column;
        self.advance()?;

        if matches!(self.peek(0)?.kind, TokenKind::RParen) {
            return self.error("Tuple must have at least one type");
        }

        types.push(self.parse_type()?);

        while !matches!(self.peek(0)?.kind, TokenKind::EOF) {
            match self.peek(0)?.kind {
                TokenKind::Comma => {
                    self.advance()?;
                    types.push(self.parse_type()?);
                }
                TokenKind::RParen => {
                    break;
                }
                _ => {
                    return self.error("Expected ',' or ')'");
                }
            }
        }

        if matches!(self.peek(0)?.kind, TokenKind::EOF) {
            return self.loc_error(paren_line, paren_col, "Unterminated tuple type");
        }

        Ok(Type::Tuple(types))
    }

    fn parse_array_type(&mut self) -> Result<Type<'a>, Error> {
        self.advance()?;

        if matches!(self.peek(0)?.kind, TokenKind::RSquare) {
            return self.error("Array must have a type");
        }

        let ty = Box::new(self.parse_type()?);

        if !matches!(self.peek(0)?.kind, TokenKind::RSquare) {
            return self.error("Unterminated array type");
        }

        Ok(Type::Array(ty))
    }
}
