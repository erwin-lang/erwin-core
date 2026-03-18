#[derive(Clone, Copy)]
pub(crate) struct Token<'a> {
    pub(crate) kind: TokenKind<'a>,
    pub(crate) line: usize,
    pub(crate) column: usize,
}

impl<'a> Token<'a> {
    pub(crate) fn new(kind: TokenKind<'a>, line: usize, column: usize) -> Self {
        Self { kind, line, column }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum TokenKind<'a> {
    Identifier(&'a str),

    // Keywords
    Var,      // Variable definition
    Node,     // Node variable definition
    Const,    // Constant variable definition
    Obj,      // Object definition for data storage
    Func,     // Function definition
    Return,   // Exit function with return value
    For,      // For loop
    While,    // While loop
    Continue, // Proceed to next iteration in loop
    Break,    // Break out of loop
    If,       // If block
    Else,     // Else block
    True,     // Boolean true
    False,    // Boolean false
    Pub,      // Public visibility modifier
    Import,   // Import another module

    // Primitive types
    Bool,
    Int8,
    Int16,
    Int32,
    Int64,
    Int128,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Uint128,
    Float32,
    Float64,
    String,
    Pointer,
    Byte,

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
    LParen,    // (
    RParen,    // )
    LSquare,   // [
    RSquare,   // ]
    LBrace,    // {
    RBrace,    // }
    Semicolon, // ;
    Colon,     // :
    Comma,     // ,
    Dot,       // .

    // Symbols
    Plus,   // +
    Minus,  // -
    Star,   // *
    Slash,  // /
    Pow,    // ^
    LPipe,  // <|
    RPipe,  // |>
    LArrow, // <-
    RArrow, // ->

    EOF, // End of file
}
