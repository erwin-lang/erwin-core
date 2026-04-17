use std::path::Path;

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
    Universal, // Universal type
    Unit,      // Single value
    Null,      // Null type
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
    pub(crate) fn from_str(value: &'a str) -> Self {
        match value {
            "Bool" => Type::Bool,
            "UInt8" => Type::Integer {
                size: IntSize::B8,
                sign: Sign::Unsigned,
            },
            "UInt16" => Type::Integer {
                size: IntSize::B16,
                sign: Sign::Unsigned,
            },
            "UInt32" => Type::Integer {
                size: IntSize::B32,
                sign: Sign::Unsigned,
            },
            "UInt64" => Type::Integer {
                size: IntSize::B64,
                sign: Sign::Unsigned,
            },
            "UInt128" => Type::Integer {
                size: IntSize::B128,
                sign: Sign::Unsigned,
            },
            "Int8" => Type::Integer {
                size: IntSize::B8,
                sign: Sign::Signed,
            },
            "Int16" => Type::Integer {
                size: IntSize::B16,
                sign: Sign::Signed,
            },
            "Int32" => Type::Integer {
                size: IntSize::B32,
                sign: Sign::Signed,
            },
            "Int64" => Type::Integer {
                size: IntSize::B64,
                sign: Sign::Signed,
            },
            "Int128" => Type::Integer {
                size: IntSize::B128,
                sign: Sign::Signed,
            },
            "URange8" => Type::IntRange {
                size: IntSize::B8,
                sign: Sign::Unsigned,
            },
            "URange16" => Type::IntRange {
                size: IntSize::B16,
                sign: Sign::Unsigned,
            },
            "URange32" => Type::IntRange {
                size: IntSize::B32,
                sign: Sign::Unsigned,
            },
            "URange64" => Type::IntRange {
                size: IntSize::B64,
                sign: Sign::Unsigned,
            },
            "URange128" => Type::IntRange {
                size: IntSize::B128,
                sign: Sign::Unsigned,
            },
            "Range8" => Type::IntRange {
                size: IntSize::B8,
                sign: Sign::Signed,
            },
            "Range16" => Type::IntRange {
                size: IntSize::B16,
                sign: Sign::Signed,
            },
            "Range32" => Type::IntRange {
                size: IntSize::B32,
                sign: Sign::Signed,
            },
            "Range64" => Type::IntRange {
                size: IntSize::B64,
                sign: Sign::Signed,
            },
            "Range128" => Type::IntRange {
                size: IntSize::B128,
                sign: Sign::Signed,
            },
            "Float32" => Type::Float {
                size: FloatSize::B32,
            },
            "Float64" => Type::Float {
                size: FloatSize::B64,
            },
            "Str" => Type::String,
            "Ptr" => Type::Pointer(Box::new(Type::Universal)),
            "Ref" => Type::Ref(Box::new(Type::Universal)),
            "Tuple" => Type::Tuple(Vec::new()),
            "Array" => Type::Array(Box::new(Type::Universal)),
            "Func" => Type::Function {
                params: Vec::new(),
                return_ty: Box::new(Type::Universal),
            },
            "Node" => Type::Node(Box::new(Type::Universal)),
            _ => Type::Custom(value),
        }
    }

    pub(crate) fn as_str(&self) -> &'a str {
        match self {
            Type::Bool => "Bool",
            Type::Integer {
                size: IntSize::B8,
                sign: Sign::Unsigned,
            } => "UInt8",
            Type::Integer {
                size: IntSize::B16,
                sign: Sign::Unsigned,
            } => "UInt16",
            Type::Integer {
                size: IntSize::B32,
                sign: Sign::Unsigned,
            } => "UInt32",
            Type::Integer {
                size: IntSize::B64,
                sign: Sign::Unsigned,
            } => "UInt64",
            Type::Integer {
                size: IntSize::B128,
                sign: Sign::Unsigned,
            } => "UInt128",
            Type::Integer {
                size: IntSize::B8,
                sign: Sign::Signed,
            } => "Int8",
            Type::Integer {
                size: IntSize::B16,
                sign: Sign::Signed,
            } => "Int16",
            Type::Integer {
                size: IntSize::B32,
                sign: Sign::Signed,
            } => "Int32",
            Type::Integer {
                size: IntSize::B64,
                sign: Sign::Signed,
            } => "Int64",
            Type::Integer {
                size: IntSize::B128,
                sign: Sign::Signed,
            } => "Int128",
            Type::IntRange {
                size: IntSize::B8,
                sign: Sign::Unsigned,
            } => "URange8",
            Type::IntRange {
                size: IntSize::B16,
                sign: Sign::Unsigned,
            } => "URange16",
            Type::IntRange {
                size: IntSize::B32,
                sign: Sign::Unsigned,
            } => "URange32",
            Type::IntRange {
                size: IntSize::B64,
                sign: Sign::Unsigned,
            } => "URange64",
            Type::IntRange {
                size: IntSize::B128,
                sign: Sign::Unsigned,
            } => "URange128",
            Type::IntRange {
                size: IntSize::B8,
                sign: Sign::Signed,
            } => "Range8",
            Type::IntRange {
                size: IntSize::B16,
                sign: Sign::Signed,
            } => "Range16",
            Type::IntRange {
                size: IntSize::B32,
                sign: Sign::Signed,
            } => "Range32",
            Type::IntRange {
                size: IntSize::B64,
                sign: Sign::Signed,
            } => "Range64",
            Type::IntRange {
                size: IntSize::B128,
                sign: Sign::Signed,
            } => "Range128",
            Type::Float {
                size: FloatSize::B32,
            } => "Float32",
            Type::Float {
                size: FloatSize::B64,
            } => "Float64",
            Type::String => "Str",
            Type::Pointer(_) => "Ptr",
            Type::Tuple(_) => "Tuple",
            Type::Array(_) => "Array",
            Type::Function { .. } => "Func",
            Type::Node(_) => "Node",
            Type::Custom(id) => id,
            _ => "",
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
