
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenKind {
    // keywords
    Do,
    End,
    Fn,
    If,
    Else,
    While,
    For,
    In,
    Data,
    Is,
    New,
    Err,
    Ok,

    // syntax
    LParen,
    RParen,
    Comma,
    Dot,
    Assign,
    Newline,

    // operators
    Bang,
    Plus,
    Hyphen,
    Star,
    FSlash,
    Mod,
    And,
    Or,
    BAnd,
    BOr,
    BXor,
    Equals,
    BangEquals,
    LessThan,
    GreaterThan,
    LessThanEquals,
    GreaterThanEquals,

    // other syntax elements
    Ident,
    String,
    Double(f64),
    Integer(i64),

    Eof,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: u64,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: String, line: u64) -> Self {
        Token {
            kind,
            lexeme,
            line,
        }
    }
}

