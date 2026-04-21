#[derive(Clone, Copy)]
pub(crate) struct Token<'a> {
    pub(crate) kind: TokenKind<'a>,
    pub(crate) line: usize,
    pub(crate) col: usize,
}

impl<'a> Token<'a> {
    pub(crate) fn new(kind: TokenKind<'a>, line: usize, column: usize) -> Self {
        Self {
            kind,
            line,
            col: column,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum TokenKind<'a> {
    Identifier(&'a str),

    // Keywords
    Var,       // Variable definition
    Mut,       // Variable assignment
    Const,     // Constant variable definition
    Node,      // Node definition
    State,     // Object state definition block
    Container, // Type generic container
    Enum,      // Sum type definition block
    Method,    // Method implementation block
    Func,      // Function definition
    Return,    // Exit function with return value
    Yield,     // Yield a value from the local scope
    Do,        // Used before control flow body
    For,       // For loop
    In,        // For loop range
    While,     // While loop
    Continue,  // Proceed to next iteration in loop
    Break,     // Break out of loop
    If,        // If block
    Else,      // Used in control flow to indicate fallback expression
    True,      // Boolean true
    False,     // Boolean false
    Pub,       // Public visibility modifier
    Import,    // Import another module
    Alias,     // Type alias

    // Primitive types
    Bool,
    Int8,
    Int16,
    Int32,
    Int64,
    Int128,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    UInt128,
    IntRange8,
    IntRange16,
    IntRange32,
    IntRange64,
    IntRange128,
    UIntRange8,
    UIntRange16,
    UIntRange32,
    UIntRange64,
    UIntRange128,
    Float32,
    Float64,
    String,

    // Values
    Number(&'a str),
    StringLiteral(&'a str),

    // Assign
    Assign, // =

    // Compares
    Equal,        // ==
    NotEqual,     // !=
    LessThan,     // <
    GreaterThan,  // >
    LessEqual,    // <=
    GreaterEqual, // >=

    // Logic gates
    Not,  // !
    Or,   // ||
    And,  // &&
    Nor,  // !|
    Nand, // !&
    Xor,  // ^^
    Xnor, // !^

    // Delimiters
    LParen,      // (
    RParen,      // )
    LSquare,     // [
    RSquare,     // ]
    LBrace,      // {
    RBrace,      // }
    Semicolon,   // ;
    Colon,       // :
    DoubleColon, // ::
    Comma,       // ,
    Dot,         // .

    // Symbols
    At,        // @
    Amp,       // &
    Plus,      // +
    Minus,     // -
    Star,      // *
    Slash,     // /
    Pow,       // ^
    RPipe,     // |>
    RArrow,    // ->
    DoubleDot, // ..

    Eof, // End of file
}
