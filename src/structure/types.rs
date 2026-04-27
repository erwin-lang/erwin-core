use std::path::Path;

use crate::structure::type_expr::TypeSymbolKind;

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Type<'a> {
    // User accessible types
    Bool,
    Integer {
        size: IntSize,
        sign: Sign,
    },
    IntRange {
        size: IntSize,
        sign: Sign,
    },
    Float {
        size: FloatSize,
    },
    String,
    Pointer(Box<Type<'a>>),
    Ref(Box<Type<'a>>),
    Tuple(Vec<Type<'a>>),
    Array(Box<Type<'a>>),
    Function {
        params: Vec<Type<'a>>,
        return_ty: Box<Type<'a>>,
    },
    Node(Box<Type<'a>>),
    Custom(&'a str),

    // Special builtin types
    Module(&'a Path),
    Unknown, // Promotable type | is_assignable(_, Unknown) = true | join_ty(Unknown, _) = _
    Unit,    // Single value
    Done,    // Control flow type | join_ty(Done, _) = Done
}

#[derive(Debug, PartialEq, Clone, Copy, PartialOrd)]
pub(crate) enum IntSize {
    B8,
    B16,
    B32,
    B64,
    B128,
}

#[derive(Debug, PartialEq, Clone, Copy, PartialOrd)]
pub(crate) enum Sign {
    Unsigned,
    Signed,
}

#[derive(Debug, PartialEq, Clone, PartialOrd)]
pub(crate) enum FloatSize {
    B32,
    B64,
}

impl<'a> Type<'a> {
    pub(crate) fn registry_id(&self) -> Option<TypeSymbolKind> {
        match self {
            Type::Bool => Some(TypeSymbolKind::Bool),
            Type::Integer {
                size: IntSize::B8,
                sign: Sign::Unsigned,
            } => Some(TypeSymbolKind::UInt8),
            Type::Integer {
                size: IntSize::B16,
                sign: Sign::Unsigned,
            } => Some(TypeSymbolKind::UInt16),
            Type::Integer {
                size: IntSize::B32,
                sign: Sign::Unsigned,
            } => Some(TypeSymbolKind::UInt32),
            Type::Integer {
                size: IntSize::B64,
                sign: Sign::Unsigned,
            } => Some(TypeSymbolKind::UInt64),
            Type::Integer {
                size: IntSize::B128,
                sign: Sign::Unsigned,
            } => Some(TypeSymbolKind::UInt128),
            Type::Integer {
                size: IntSize::B8,
                sign: Sign::Signed,
            } => Some(TypeSymbolKind::Int8),
            Type::Integer {
                size: IntSize::B16,
                sign: Sign::Signed,
            } => Some(TypeSymbolKind::Int16),
            Type::Integer {
                size: IntSize::B32,
                sign: Sign::Signed,
            } => Some(TypeSymbolKind::Int32),
            Type::Integer {
                size: IntSize::B64,
                sign: Sign::Signed,
            } => Some(TypeSymbolKind::Int64),
            Type::Integer {
                size: IntSize::B128,
                sign: Sign::Signed,
            } => Some(TypeSymbolKind::Int128),
            Type::IntRange {
                size: IntSize::B8,
                sign: Sign::Unsigned,
            } => Some(TypeSymbolKind::URange8),
            Type::IntRange {
                size: IntSize::B16,
                sign: Sign::Unsigned,
            } => Some(TypeSymbolKind::URange16),
            Type::IntRange {
                size: IntSize::B32,
                sign: Sign::Unsigned,
            } => Some(TypeSymbolKind::URange32),
            Type::IntRange {
                size: IntSize::B64,
                sign: Sign::Unsigned,
            } => Some(TypeSymbolKind::URange64),
            Type::IntRange {
                size: IntSize::B128,
                sign: Sign::Unsigned,
            } => Some(TypeSymbolKind::URange128),
            Type::IntRange {
                size: IntSize::B8,
                sign: Sign::Signed,
            } => Some(TypeSymbolKind::Range8),
            Type::IntRange {
                size: IntSize::B16,
                sign: Sign::Signed,
            } => Some(TypeSymbolKind::Range16),
            Type::IntRange {
                size: IntSize::B32,
                sign: Sign::Signed,
            } => Some(TypeSymbolKind::Range32),
            Type::IntRange {
                size: IntSize::B64,
                sign: Sign::Signed,
            } => Some(TypeSymbolKind::Range64),
            Type::IntRange {
                size: IntSize::B128,
                sign: Sign::Signed,
            } => Some(TypeSymbolKind::Range128),
            Type::Float {
                size: FloatSize::B32,
            } => Some(TypeSymbolKind::Float32),
            Type::Float {
                size: FloatSize::B64,
            } => Some(TypeSymbolKind::Float64),
            Type::String => Some(TypeSymbolKind::Str),
            Type::Pointer(_) => Some(TypeSymbolKind::Ptr),
            Type::Tuple(_) => Some(TypeSymbolKind::Tuple),
            Type::Array(_) => Some(TypeSymbolKind::Array),
            Type::Function { .. } => Some(TypeSymbolKind::Func),
            Type::Node(_) => Some(TypeSymbolKind::Node),
            Type::Custom(id) => Some(TypeSymbolKind::Custom(id)),
            _ => None,
        }
    }

    pub(crate) fn elem_type(&self) -> Option<Self> {
        match self {
            Type::IntRange { size, sign } => Some(Type::Integer {
                size: *size,
                sign: *sign,
            }),
            Type::Array(ty) => Some((**ty).clone()),
            Type::String => Some(Type::Integer {
                size: IntSize::B8,
                sign: Sign::Unsigned,
            }),
            Type::Ref(ty) | Type::Pointer(ty) => {
                ty.elem_type().map(|elem_ty| Type::Ref(Box::new(elem_ty)))
            }
            _ => None,
        }
    }
}
