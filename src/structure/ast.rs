use crate::structure::types::Type;

#[derive(Clone, Debug)]
pub(crate) struct Statement<'a> {
    pub(crate) kind: StatementKind<'a>,
    pub(crate) line: usize,
    pub(crate) col: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct Expr<'a> {
    pub(crate) kind: ExprKind<'a>,
    pub(crate) line: usize,
    pub(crate) col: usize,
}

#[derive(Clone, Debug)]
pub(crate) enum StatementKind<'a> {
    Import {
        alias: Option<&'a str>,
        path: Vec<&'a str>,
    },
    VarDeclare {
        visibility: Visibility,
        kind: VarKind,
        id: &'a str,
        ty: Option<Type<'a>>,
        value: Expr<'a>,
    },
    VarAssign {
        id: Expr<'a>,
        value: Expr<'a>,
    },
    Node {
        visibility: Visibility,
        id: &'a str,
        ty: Type<'a>,
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
    Container {
        visibility: Visibility,
        id: &'a str,
        types: Expr<'a>,
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
    Alias {
        alias_id: &'a str,
        ty: Type<'a>,
    },
    Expr(Expr<'a>),
}

#[derive(Clone, Debug)]
pub(crate) enum VarKind {
    Const,
    Var,
}

#[derive(Clone, Debug)]
pub(crate) enum Visibility {
    Pub,
    Priv,
}

#[derive(Clone, Debug)]
pub(crate) struct Param<'a> {
    pub(crate) id: &'a str,
    pub(crate) ty: Type<'a>,
}

#[derive(Clone, Debug)]
pub(crate) struct Field<'a> {
    pub(crate) visibility: Visibility,
    pub(crate) id: &'a str,
    pub(crate) ty: Type<'a>,
}

#[derive(Clone, Debug)]
pub(crate) struct InstanceField<'a> {
    pub(crate) id: &'a str,
    pub(crate) value: Expr<'a>,
}

#[derive(Clone, Debug)]
pub(crate) struct Variant<'a> {
    pub(crate) id: &'a str,
    pub(crate) data: Vec<Type<'a>>,
}

#[derive(Clone, Debug)]
pub(crate) enum ExprKind<'a> {
    Number(&'a str),
    String(&'a str),
    Bool(bool),
    Identifier(&'a str),
    StaticAccess {
        target: Box<Expr<'a>>,
        member: &'a str,
    },
    MemberAccess {
        target: Box<Expr<'a>>,
        member: &'a str,
    },
    Tuple(Vec<Expr<'a>>),
    Array(Vec<Expr<'a>>),
    Block(Vec<Statement<'a>>),
    Return(Box<Expr<'a>>),
    Yield(Box<Expr<'a>>),
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
        elem: &'a str,
        iter: Box<Expr<'a>>,
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
        params: Vec<Param<'a>>,
        body: Box<Expr<'a>>,
    },
}

#[derive(Clone, Debug)]
pub(crate) enum UnaryOp {
    Not,
    Minus,
    Ref,
    Deref,
}

#[derive(Clone, Debug)]
pub(crate) enum BinaryOp {
    Pow,

    Mult,
    Div,

    Add,
    Sub,

    Range,

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

    RPipe,
}
