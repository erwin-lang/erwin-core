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
        let mut ty = self.parse_type_prefix()?;
        ty = self.parse_type_postfix(ty)?;
        Ok(ty)
    }

    fn parse_type_prefix(&mut self) -> Result<Type<'a>, Error> {
        match self.peek(0)?.kind {
            TokenKind::Amp => {
                self.advance()?;
                let inner = self.parse_type()?;
                Ok(Type::Ref(Box::new(inner)))
            }
            TokenKind::At => {
                self.advance()?;
                let inner = self.parse_type()?;
                Ok(Type::Node(Box::new(inner)))
            }
            _ => self.parse_base_type(),
        }
    }

    fn parse_base_type(&mut self) -> Result<Type<'a>, Error> {
        match self.peek(0)?.kind {
            TokenKind::Bool => {
                self.advance()?;
                Ok(Type::Bool)
            }
            TokenKind::Int8 => {
                self.advance()?;
                Ok(Type::Integer {
                    size: IntSize::B8,
                    sign: Sign::Signed,
                })
            }
            TokenKind::Int16 => {
                self.advance()?;
                Ok(Type::Integer {
                    size: IntSize::B16,
                    sign: Sign::Signed,
                })
            }
            TokenKind::Int32 => {
                self.advance()?;
                Ok(Type::Integer {
                    size: IntSize::B32,
                    sign: Sign::Signed,
                })
            }
            TokenKind::Int64 => {
                self.advance()?;
                Ok(Type::Integer {
                    size: IntSize::B64,
                    sign: Sign::Signed,
                })
            }
            TokenKind::Int128 => {
                self.advance()?;
                Ok(Type::Integer {
                    size: IntSize::B128,
                    sign: Sign::Signed,
                })
            }
            TokenKind::UInt8 => {
                self.advance()?;
                Ok(Type::Integer {
                    size: IntSize::B8,
                    sign: Sign::Unsigned,
                })
            }
            TokenKind::UInt16 => {
                self.advance()?;
                Ok(Type::Integer {
                    size: IntSize::B16,
                    sign: Sign::Unsigned,
                })
            }
            TokenKind::UInt32 => {
                self.advance()?;
                Ok(Type::Integer {
                    size: IntSize::B32,
                    sign: Sign::Unsigned,
                })
            }
            TokenKind::UInt64 => {
                self.advance()?;
                Ok(Type::Integer {
                    size: IntSize::B64,
                    sign: Sign::Unsigned,
                })
            }
            TokenKind::UInt128 => {
                self.advance()?;
                Ok(Type::Integer {
                    size: IntSize::B128,
                    sign: Sign::Unsigned,
                })
            }
            TokenKind::IntRange8 => {
                self.advance()?;
                Ok(Type::IntRange {
                    size: IntSize::B8,
                    sign: Sign::Signed,
                })
            }
            TokenKind::IntRange16 => {
                self.advance()?;
                Ok(Type::IntRange {
                    size: IntSize::B16,
                    sign: Sign::Signed,
                })
            }
            TokenKind::IntRange32 => {
                self.advance()?;
                Ok(Type::IntRange {
                    size: IntSize::B32,
                    sign: Sign::Signed,
                })
            }
            TokenKind::IntRange64 => {
                self.advance()?;
                Ok(Type::IntRange {
                    size: IntSize::B64,
                    sign: Sign::Signed,
                })
            }
            TokenKind::IntRange128 => {
                self.advance()?;
                Ok(Type::IntRange {
                    size: IntSize::B128,
                    sign: Sign::Signed,
                })
            }
            TokenKind::UIntRange8 => {
                self.advance()?;
                Ok(Type::IntRange {
                    size: IntSize::B8,
                    sign: Sign::Unsigned,
                })
            }
            TokenKind::UIntRange16 => {
                self.advance()?;
                Ok(Type::IntRange {
                    size: IntSize::B16,
                    sign: Sign::Unsigned,
                })
            }
            TokenKind::UIntRange32 => {
                self.advance()?;
                Ok(Type::IntRange {
                    size: IntSize::B32,
                    sign: Sign::Unsigned,
                })
            }
            TokenKind::UIntRange64 => {
                self.advance()?;
                Ok(Type::IntRange {
                    size: IntSize::B64,
                    sign: Sign::Unsigned,
                })
            }
            TokenKind::UIntRange128 => {
                self.advance()?;
                Ok(Type::IntRange {
                    size: IntSize::B128,
                    sign: Sign::Unsigned,
                })
            }
            TokenKind::Float32 => {
                self.advance()?;
                Ok(Type::Float {
                    size: FloatSize::B32,
                })
            }
            TokenKind::Float64 => {
                self.advance()?;
                Ok(Type::Float {
                    size: FloatSize::B64,
                })
            }
            TokenKind::String => {
                self.advance()?;
                Ok(Type::String)
            }
            TokenKind::LParen => self.parse_tuple_type(),
            TokenKind::LSquare => self.parse_array_type(),
            TokenKind::Identifier(left) => {
                self.advance()?;

                if matches!(self.peek(0)?.kind, TokenKind::DoubleDot) {
                    self.advance()?;

                    if let TokenKind::Identifier(right) = self.peek(0)?.kind {
                        self.advance()?;

                        return Ok(Type::Custom {
                            module: Some(left),
                            id: right,
                        });
                    }
                }

                Ok(Type::Custom {
                    module: None,
                    id: left,
                })
            }
            _ => self.error("Invalid type"),
        }
    }

    fn parse_type_postfix(&mut self, mut base: Type<'a>) -> Result<Type<'a>, Error> {
        loop {
            match self.peek(0)?.kind {
                TokenKind::Star => {
                    self.advance()?;
                    base = Type::Pointer(Box::new(base));
                }
                TokenKind::RArrow => {
                    self.advance()?;
                    let return_ty = self.parse_type()?;
                    let params = match base {
                        Type::Tuple(types) => types,
                        other => vec![other],
                    };
                    base = Type::Function {
                        params,
                        return_ty: Box::new(return_ty),
                    };

                    return Ok(base);
                }
                _ => break,
            }
        }

        Ok(base)
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
