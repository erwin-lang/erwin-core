use crate::structure::types::Type;

pub(crate) struct Statement<'a> {
    pub(crate) kind: StatementKind<'a>,
    pub(crate) line: usize,
    pub(crate) col: usize,
}

pub(crate) struct Expr<'a> {
    pub(crate) kind: ExprKind<'a>,
    pub(crate) line: usize,
    pub(crate) col: usize,
}

pub(crate) enum StatementKind<'a> {
    Import {
        path: Vec<&'a str>,
    },
    Var {
        visibility: Visibility,
        kind: VarKind,
        id: &'a str,
        ty: Option<Type<'a>>,
        value: Expr<'a>,
    },
    Func {
        visibility: Visibility,
        id: &'a str,
        params: Vec<Param<'a>>,
        ty: Type<'a>,
        body: Expr<'a>,
    },
    State {
        visibility: Visibility,
        id: &'a str,
        fields: Vec<Field<'a>>,
    },
    Method {
        id: &'a str,
        methods: Expr<'a>,
    },
    Enum {
        visibility: Visibility,
        id: &'a str,
        variants: Vec<Variant<'a>>,
    },
    Expr(Expr<'a>),
}

pub(crate) enum VarKind {
    Const,
    Var,
    Node,
}

pub(crate) enum Visibility {
    Pub,
    Priv,
}

pub(crate) struct Param<'a> {
    pub(crate) id: &'a str,
    pub(crate) ty: Type<'a>,
}

pub(crate) struct Field<'a> {
    pub(crate) visibility: Visibility,
    pub(crate) id: &'a str,
    pub(crate) ty: Type<'a>,
}

pub(crate) struct InstanceField<'a> {
    pub(crate) id: &'a str,
    pub(crate) value: Expr<'a>,
}

pub(crate) struct Variant<'a> {
    pub(crate) id: &'a str,
    pub(crate) data: Vec<Type<'a>>,
}

pub(crate) enum ExprKind<'a> {
    Number(&'a str),
    String(&'a str),
    Bool(bool),
    Identifier(&'a str),
    Path(Vec<&'a str>),
    MemberAccess {
        target: Box<Expr<'a>>,
        member: &'a str,
    },
    Tuple(Vec<Expr<'a>>),
    Array(Vec<Expr<'a>>),
    Block(Vec<Statement<'a>>),
    Return(Box<Expr<'a>>),
    Break,
    Continue,
    StateInstance {
        id: &'a str,
        fields: Vec<InstanceField<'a>>,
    },

    Call {
        base: Box<Expr<'a>>,
        args: Vec<Expr<'a>>,
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

    For {
        iter: Box<Expr<'a>>,
        range: Box<Expr<'a>>,
        do_body: Box<Expr<'a>>,
        else_body: Option<Box<Expr<'a>>>,
    },
    While {
        condition: Box<Expr<'a>>,
        do_body: Box<Expr<'a>>,
        else_body: Option<Box<Expr<'a>>>,
    },
    If {
        condition: Box<Expr<'a>>,
        do_body: Box<Expr<'a>>,
        else_body: Option<Box<Expr<'a>>>,
    },

    Lambda {
        params: Vec<&'a str>,
        body: Box<Expr<'a>>,
    },
}

pub(crate) enum UnaryOp {
    Not,
    Minus,
}

pub(crate) enum BinaryOp {
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
