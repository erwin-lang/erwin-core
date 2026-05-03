use crate::structure::checker::module_table::TypeSymbolKind;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Literal<'a> {
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    UInt128(u128),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Int128(i128),
    Float32(f32),
    Float64(f64),
    Str(&'a str),
    Bool(&'a bool),
}

impl<'a> Literal<'a> {
    pub(crate) fn type_symbol_id(&self) -> TypeSymbolKind<'a> {
        match self {
            Literal::UInt8(_) => TypeSymbolKind::UInt8,
            Literal::UInt16(_) => TypeSymbolKind::UInt16,
            Literal::UInt32(_) => TypeSymbolKind::UInt32,
            Literal::UInt64(_) => TypeSymbolKind::UInt64,
            Literal::UInt128(_) => TypeSymbolKind::UInt128,
            Literal::Int8(_) => TypeSymbolKind::Int8,
            Literal::Int16(_) => TypeSymbolKind::Int16,
            Literal::Int32(_) => TypeSymbolKind::Int32,
            Literal::Int64(_) => TypeSymbolKind::Int64,
            Literal::Int128(_) => TypeSymbolKind::Int128,
            Literal::Float32(_) => TypeSymbolKind::Float32,
            Literal::Float64(_) => TypeSymbolKind::Float64,
            Literal::Str(_) => TypeSymbolKind::Str,
            Literal::Bool(_) => TypeSymbolKind::Bool,
        }
    }
}
