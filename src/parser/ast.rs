pub enum Statement {
    Import {
        module: String,
    },
    VarDeclare {
        visibility: Visibility,
        kind: VarKind,
        identifier: String,
        ty: Option<Type>,
        value: Expr,
    },
    NodeDeclare {
        visibility: Visibility,
        identifier: String,
        ty: Option<Type>,
        value: Expr,
        body: Option<Vec<Statement>>,
    },
    Func {
        visibility: Visibility,
        identifier: String,
        params: Option<Vec<Param>>,
        body: Vec<Statement>,
    },
    For {
        iter: String,
        start: Expr,
        end: Expr,
        step: Option<Expr>,
        body: Vec<Statement>,
        else_body: Option<Else>,
    },
    While {
        condition: Expr,
        body: Vec<Statement>,
        else_body: Option<Else>,
    },
    If {
        condition: Expr,
        then_body: Vec<Statement>,
        else_body: Option<Else>,
    },
    Return {
        value: Expr,
    },
    Break,
    Continue,
    Obj {
        identifier: String,
        body: Vec<Field>,
    },
    Expr(Expr),
}

pub enum VarKind {
    Const,
    Var,
}

pub enum Visibility {
    Pub,
    Priv,
}

pub enum Else {
    Else(Vec<Statement>),
    ElseIf {
        condition: Expr,
        then_body: Vec<Statement>,
        else_body: Option<Box<Else>>,
    },
    ElseWhile {
        condition: Expr,
        then_body: Vec<Statement>,
        else_body: Option<Box<Else>>,
    },
    ElseFor {
        iter: String,
        start: Expr,
        end: Expr,
        step: Option<Expr>,
        then_body: Vec<Statement>,
        else_body: Option<Box<Else>>,
    },
}

pub struct Param {
    pub identifier: String,
    pub ty: Type,
}

pub struct Field {
    pub identifier: String,
    pub ty: Type,
}

pub enum Type {
    Bool,
    Integer { size: IntSize, signed: Sign },
    Float { size: FloatSize },
    String,
    Pointer(Box<Type>),
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

pub enum Expr {
    Number(String),
    String(String),
    Bool(bool),
    Identifier(String),
    Unary {
        op: UnaryOp,
        right: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Call {
        identifier: String,
        args: Option<Vec<Expr>>,
    },
}

pub enum UnaryOp {
    Not,
    Minus,
}

pub enum BinaryOp {
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,
    Or,
    And,
    Nor,
    Nand,
    Xor,
    Xnor,
    Add,
    Sub,
    Mult,
    Div,
    Pow,
    LPipe,
    RPipe,
    LArrow,
    RArrow,
}
