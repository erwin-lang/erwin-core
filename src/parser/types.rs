use crate::{
    error::Error,
    parser::Parser,
    structure::{
        token::TokenKind,
        types::{FloatSize, IntSize, Sign, Type},
    },
};

impl<'a> Parser<'a> {
    pub(super) fn parse_type(&mut self) -> Result<Type<'a>, Error> {
        let mut base_type = match self.peek(0)?.kind {
            TokenKind::Bool => {
                self.advance()?;
                Type::Bool
            }
            TokenKind::Int8 => {
                self.advance()?;
                Type::Integer {
                    size: IntSize::B8,
                    sign: Sign::Signed,
                }
            }
            TokenKind::Int16 => {
                self.advance()?;
                Type::Integer {
                    size: IntSize::B16,
                    sign: Sign::Signed,
                }
            }
            TokenKind::Int32 => {
                self.advance()?;
                Type::Integer {
                    size: IntSize::B32,
                    sign: Sign::Signed,
                }
            }
            TokenKind::Int64 => {
                self.advance()?;
                Type::Integer {
                    size: IntSize::B64,
                    sign: Sign::Signed,
                }
            }
            TokenKind::Uint8 => {
                self.advance()?;
                Type::Integer {
                    size: IntSize::B8,
                    sign: Sign::Unsigned,
                }
            }
            TokenKind::Uint16 => {
                self.advance()?;
                Type::Integer {
                    size: IntSize::B16,
                    sign: Sign::Unsigned,
                }
            }
            TokenKind::Uint32 => {
                self.advance()?;
                Type::Integer {
                    size: IntSize::B32,
                    sign: Sign::Unsigned,
                }
            }
            TokenKind::Uint64 => {
                self.advance()?;
                Type::Integer {
                    size: IntSize::B64,
                    sign: Sign::Unsigned,
                }
            }
            TokenKind::Float32 => {
                self.advance()?;
                Type::Float {
                    size: FloatSize::B32,
                }
            }
            TokenKind::Float64 => {
                self.advance()?;
                Type::Float {
                    size: FloatSize::B64,
                }
            }
            TokenKind::String => {
                self.advance()?;
                Type::String
            }
            TokenKind::LParen => self.parse_tuple_type()?,
            TokenKind::LSquare => self.parse_array_type()?,
            TokenKind::Byte => {
                self.advance()?;
                Type::Byte
            }
            TokenKind::Identifier(_) => Type::Custom(self.parse_path()?),
            _ => return self.error("Invalid type"),
        };

        while matches!(self.peek(0)?.kind, TokenKind::Star) {
            self.advance()?;
            base_type = Type::Pointer(Box::new(base_type));
        }

        if matches!(self.peek(0)?.kind, TokenKind::RArrow) {
            self.advance()?;

            let return_ty = self.parse_type()?;

            let params = match base_type {
                Type::Tuple(types) => types,
                other => vec![other],
            };

            base_type = Type::Function {
                params,
                return_ty: Box::new(return_ty),
            };
        }

        Ok(base_type)
    }

    fn parse_tuple_type(&mut self) -> Result<Type<'a>, Error> {
        self.advance()?;

        let mut types = self.parse_comma_separated(|p| p.parse_type())?;
        self.consume(TokenKind::RParen, "Expected ')'")?;

        match types.len() {
            1 => Ok(types.remove(0)),
            _ => Ok(Type::Tuple(types)),
        }
    }

    fn parse_array_type(&mut self) -> Result<Type<'a>, Error> {
        self.advance()?;

        if matches!(self.peek(0)?.kind, TokenKind::RSquare) {
            return self.error("Array must have a type");
        }

        let ty = Box::new(self.parse_type()?);

        self.consume(TokenKind::RSquare, "Expected ']'")?;

        Ok(Type::Array(ty))
    }
}
