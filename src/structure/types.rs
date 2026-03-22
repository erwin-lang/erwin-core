pub(crate) enum Type<'a> {
    Bool,
    Integer {
        size: IntSize,
        sign: Sign,
    },
    Float {
        size: FloatSize,
    },
    String,
    Pointer(Box<Type<'a>>),
    Tuple(Vec<Type<'a>>),
    Array(Box<Type<'a>>),
    Byte,
    Function {
        params: Vec<Type<'a>>,
        return_ty: Box<Type<'a>>,
    },
    Node(Box<Type<'a>>),
    Custom(Vec<&'a str>),
}

pub(crate) enum IntSize {
    B8,
    B16,
    B32,
    B64,
    B128,
}

pub(crate) enum Sign {
    Signed,
    Unsigned,
}

pub(crate) enum FloatSize {
    B32,
    B64,
}
