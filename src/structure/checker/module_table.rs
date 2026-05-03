use std::{collections::HashMap, path::Path};

use crate::structure::parser::ast::Visibility;

#[derive(Debug, Clone)]
pub(crate) struct ModuleTable<'a> {
    pub(crate) scopes: Vec<Scope<'a>>,
    pub(crate) type_symbols: HashMap<&'a str, &'a TypeSymbol<'a>>,
    pub(crate) containers: HashMap<&'a str, &'a Container<'a>>,
}

#[derive(Debug, Clone)]
pub(crate) struct Scope<'a> {
    pub(crate) parent: Option<usize>,
    pub(crate) scope_symbols: HashMap<&'a str, &'a ScopeSymbol<'a>>,
}

#[derive(Debug, Clone)]
pub(crate) struct ScopeSymbol<'a> {
    pub(crate) id: &'a str,
    pub(crate) module: &'a Path,
    pub(crate) scope_index: Option<usize>,
    pub(crate) visibility: &'a Visibility,
    pub(crate) ty: &'a Type<'a>,
    pub(crate) is_static_member: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct TypeSymbol<'a> {
    pub(crate) module: &'a Path,
    pub(crate) visibility: &'a Visibility,
    pub(crate) kind: TypeSymbolKind<'a>,
    pub(crate) members: HashMap<&'a str, &'a ScopeSymbol<'a>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum TypeSymbolKind<'a> {
    Bool,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    UInt128,
    Int8,
    Int16,
    Int32,
    Int64,
    Int128,
    URange8,
    URange16,
    URange32,
    URange64,
    URange128,
    Range8,
    Range16,
    Range32,
    Range64,
    Range128,
    Float32,
    Float64,
    Str,
    Ptr,
    Ref,
    Tuple,
    Array,
    Func,
    Node,
    Custom(&'a str),
}

impl<'a> TypeSymbolKind<'a> {
    pub(crate) fn as_str(&self) -> &'a str {
        match self {
            TypeSymbolKind::Bool => "Bool",
            TypeSymbolKind::UInt8 => "UInt8",
            TypeSymbolKind::UInt16 => "UInt16",
            TypeSymbolKind::UInt32 => "UInt32",
            TypeSymbolKind::UInt64 => "UInt64",
            TypeSymbolKind::UInt128 => "UInt128",
            TypeSymbolKind::Int8 => "Int8",
            TypeSymbolKind::Int16 => "Int16",
            TypeSymbolKind::Int32 => "Int32",
            TypeSymbolKind::Int64 => "Int64",
            TypeSymbolKind::Int128 => "Int128",
            TypeSymbolKind::URange8 => "URange8",
            TypeSymbolKind::URange16 => "URange16",
            TypeSymbolKind::URange32 => "URange32",
            TypeSymbolKind::URange64 => "URange64",
            TypeSymbolKind::URange128 => "URange128",
            TypeSymbolKind::Range8 => "Range8",
            TypeSymbolKind::Range16 => "Range16",
            TypeSymbolKind::Range32 => "Range32",
            TypeSymbolKind::Range64 => "Range64",
            TypeSymbolKind::Range128 => "Range128",
            TypeSymbolKind::Float32 => "Float32",
            TypeSymbolKind::Float64 => "Float64",
            TypeSymbolKind::Str => "Str",
            TypeSymbolKind::Ptr => "Ptr",
            TypeSymbolKind::Ref => "Ref",
            TypeSymbolKind::Tuple => "Tuple",
            TypeSymbolKind::Array => "Array",
            TypeSymbolKind::Func => "Func",
            TypeSymbolKind::Node => "Node",
            TypeSymbolKind::Custom(id) => *id,
        }
    }

    pub(crate) fn from_str(s: &'a str) -> Self {
        match s {
            "Bool" => TypeSymbolKind::Bool,
            "UInt8" => TypeSymbolKind::UInt8,
            "UInt16" => TypeSymbolKind::UInt16,
            "UInt32" => TypeSymbolKind::UInt32,
            "UInt64" => TypeSymbolKind::UInt64,
            "UInt128" => TypeSymbolKind::UInt128,
            "Int8" => TypeSymbolKind::Int8,
            "Int16" => TypeSymbolKind::Int16,
            "Int32" => TypeSymbolKind::Int32,
            "Int64" => TypeSymbolKind::Int64,
            "Int128" => TypeSymbolKind::Int128,
            "URange8" => TypeSymbolKind::URange8,
            "URange16" => TypeSymbolKind::URange16,
            "URange32" => TypeSymbolKind::URange32,
            "URange64" => TypeSymbolKind::URange64,
            "URange128" => TypeSymbolKind::URange128,
            "Range8" => TypeSymbolKind::Range8,
            "Range16" => TypeSymbolKind::Range16,
            "Range32" => TypeSymbolKind::Range32,
            "Range64" => TypeSymbolKind::Range64,
            "Range128" => TypeSymbolKind::Range128,
            "Float32" => TypeSymbolKind::Float32,
            "Float64" => TypeSymbolKind::Float64,
            "Str" => TypeSymbolKind::Str,
            "Ptr" => TypeSymbolKind::Ptr,
            "Ref" => TypeSymbolKind::Ref,
            "Tuple" => TypeSymbolKind::Tuple,
            "Array" => TypeSymbolKind::Array,
            "Func" => TypeSymbolKind::Func,
            "Node" => TypeSymbolKind::Node,
            id => TypeSymbolKind::Custom(id),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Container<'a> {
    pub(crate) id: &'a str,
    pub(crate) module: &'a Path,
    pub(crate) visibility: &'a Visibility,
    pub(crate) type_symbols: Vec<&'a str>,
}

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

#[derive(Debug, PartialEq, Clone, Copy, PartialOrd)]
pub(crate) enum FloatSize {
    B32,
    B64,
}

impl<'a> Type<'a> {
    pub(crate) fn type_symbol_id(&self) -> Option<TypeSymbolKind> {
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
