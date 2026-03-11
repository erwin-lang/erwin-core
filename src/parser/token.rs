pub enum Token {
    // Identifiers
    Identifier(String), // User defined identifier
    Obj,                // Object definition for data storage
    Func,               // Function definition
    Return,             // Exit function with return value
    For,                // For loop
    While,              // While loop
    Continue,           // Proceed to next iteration in loop
    Break,              // Break out of loop
    If,                 // If block
    Else,               // Else block
    True,               // Boolean true
    False,              // Boolean false

    // Values
    Number(String),
    String(String),

    // Assigns
    Assign,     // =
    NodeAssign, // :=

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
