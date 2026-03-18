pub(crate) enum Statement<'a> {
    Import {
        path: Vec<&'a str>,
    },
    VarDeclare {
        visibility: Visibility,
        kind: VarKind,
        identifier: &'a str,
        ty: Option<Type<'a>>,
        value: Expr<'a>,
    },
    NodeDeclare {
        visibility: Visibility,
        identifier: &'a str,
        ty: Option<Type<'a>>,
        value: Expr<'a>,
        body: Option<Vec<Statement<'a>>>,
    },
    Func {
        visibility: Visibility,
        identifier: &'a str,
        params: Option<Vec<Param<'a>>>,
        return_ty: Option<Type<'a>>,
        body: Vec<Statement<'a>>,
    },
    For {
        iter: &'a str,
        start: Expr<'a>,
        end: Expr<'a>,
        step: Option<Expr<'a>>,
        body: Vec<Statement<'a>>,
        else_body: Option<Else<'a>>,
    },
    While {
        condition: Expr<'a>,
        body: Vec<Statement<'a>>,
        else_body: Option<Else<'a>>,
    },
    If {
        condition: Expr<'a>,
        then_body: Vec<Statement<'a>>,
        else_body: Option<Else<'a>>,
    },
    Return {
        value: Expr<'a>,
    },
    Break,
    Continue,
    Obj {
        identifier: &'a str,
        body: Vec<Field<'a>>,
    },
    Expr(Expr<'a>),
}

pub enum VarKind {
    Const,
    Var,
}

pub enum Visibility {
    Pub,
    Priv,
}

pub enum Else<'a> {
    Else(Vec<Statement<'a>>),
    ElseIf {
        condition: Expr<'a>,
        then_body: Vec<Statement<'a>>,
        else_body: Option<Box<Else<'a>>>,
    },
    ElseWhile {
        condition: Expr<'a>,
        then_body: Vec<Statement<'a>>,
        else_body: Option<Box<Else<'a>>>,
    },
    ElseFor {
        iter: &'a str,
        start: Expr<'a>,
        end: Expr<'a>,
        step: Option<Expr<'a>>,
        then_body: Vec<Statement<'a>>,
        else_body: Option<Box<Else<'a>>>,
    },
}

pub struct Param<'a> {
    pub identifier: &'a str,
    pub ty: Type<'a>,
}

pub struct Field<'a> {
    pub identifier: &'a str,
    pub ty: Type<'a>,
}

pub enum Type<'a> {
    Bool,
    Integer { size: IntSize, sign: Sign },
    Float { size: FloatSize },
    String,
    Pointer(Box<Type<'a>>),
    Tuple(Vec<Type<'a>>),
    Array(Box<Type<'a>>),
    Byte,
    Custom(&'a str),
}

pub enum IntSize {
    B8,
    B16,
    B32,
    B64,
    B128,
}

pub enum Sign {
    Signed,
    Unsigned,
}

pub enum FloatSize {
    B32,
    B64,
}

pub enum Expr<'a> {
    Number(&'a str),
    String(&'a str),
    Bool(bool),
    Identifier(&'a str),
    Path(Vec<&'a str>),
    Tuple(Vec<Expr<'a>>),
    Array(Vec<Expr<'a>>),
    Call {
        base: Box<Expr<'a>>,
        args: Option<Vec<Expr<'a>>>,
    },
    Unary {
        op: UnaryOp,
        right: Box<Expr<'a>>,
    },
    Binary {
        left: Box<Expr<'a>>,
        op: BinaryOp,
        right: Box<Expr<'a>>,
    },
    Lambda {
        param: &'a str,
        body: Box<Expr<'a>>,
    },
}

pub enum UnaryOp {
    Not,
    Minus,
}

pub enum BinaryOp {
    Pow,

    Mult,
    Div,

    Add,
    Sub,

    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,

    Equal,
    NotEqual,

    And,
    Nand,

    Xor,
    Xnor,

    Or,
    Nor,

    LPipe,
    RPipe,
}
