use crate::error::{ErrorKind, PiccoloError};

use core::fmt;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenKind {
    // keywords
    Do,     // do
    End,    // end
    Fn,     // fn
    If,     // if
    Else,   // else
    While,  // while
    For,    // for
    In,     // in
    Data,   // data
    Let,    // let
    Is,     // is
    Me,     // me
    New,    // new
    Err,    // err
    Retn,   // retn
    Assert, // nil
    Nil,    // nil

    // syntax
    LeftBracket,  // [
    RightBracket, // ]
    LeftParen,    // (
    RightParen,   // )
    // braces?
    Comma,          // ,
    Period,         // .
    ExclusiveRange, // ..
    InclusiveRange, // ...
    Assign,         // =

    // operators
    Not,          // !
    Plus,         // +
    Minus,        // -
    Multiply,     // *
    Divide,       // /
    Modulo,       // %
    LogicalAnd,   // &&
    LogicalOr,    // ||
    BitwiseAnd,   // &
    BitwiseOr,    // |
    BitwiseXor,   // ^
    Equal,        // ==
    NotEqual,     // !=
    Less,         // <
    Greater,      // >
    LessEqual,    // <=
    GreaterEqual, // >=
    ShiftLeft,    // <<
    ShiftRight,   // >>

    // other syntax elements
    Identifier,
    String,
    True,
    False,
    Double(f64),
    Integer(i64),

    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token<'a> {
    pub(crate) kind: TokenKind,
    pub(crate) lexeme: &'a str,
    pub(crate) line: usize,
}

impl<'a> Token<'a> {
    pub fn new(kind: TokenKind, lexeme: &'a str, line: usize) -> Self {
        Token { kind, lexeme, line }
    }
}

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            TokenKind::Identifier => write!(f, "{}", self.lexeme),
            TokenKind::String => write!(f, "string \"{}\"", self.lexeme),
            TokenKind::Double(d) => write!(f, "double {}", d),
            TokenKind::Integer(i) => write!(f, "integer {}", i),
            k => write!(f, "{:?}", k),
        }
    }
}

#[cfg(feature = "pc-debug")]
pub fn print_tokens(tokens: &[Token]) {
    let mut previous_line = 0;
    for token in tokens.iter() {
        println!(
            "{} {:?}{}",
            if token.line != previous_line {
                previous_line = token.line;
                format!("{:>4}", token.line)
            } else {
                "   |".into()
            },
            token.kind,
            if token.kind == TokenKind::Identifier {
                format!(" {}", token.lexeme)
            } else {
                "".into()
            }
        );
    }
}

fn into_keyword(s: &str) -> Option<TokenKind> {
    match s {
        "do" => Some(TokenKind::Do),
        "end" => Some(TokenKind::End),
        "fn" => Some(TokenKind::Fn),
        "if" => Some(TokenKind::If),
        "else" => Some(TokenKind::Else),
        "while" => Some(TokenKind::While),
        "for" => Some(TokenKind::For),
        "in" => Some(TokenKind::In),
        "data" => Some(TokenKind::Data),
        "let" => Some(TokenKind::Let),
        "is" => Some(TokenKind::Is),
        "me" => Some(TokenKind::Me),
        "new" => Some(TokenKind::New),
        "err" => Some(TokenKind::Err),
        "retn" => Some(TokenKind::Retn),
        "assert" => Some(TokenKind::Assert),
        "true" => Some(TokenKind::True),
        "false" => Some(TokenKind::False),
        "nil" => Some(TokenKind::Nil),
        _ => None,
    }
}

/// Converts a piccolo source into a list of tokens.
pub struct Scanner<'a> {
    source: &'a [u8],
    tokens: Vec<Token<'a>>,
    start: usize,
    current: usize,
    line: usize,
}

impl<'a> Scanner<'a> {
    /// Create a new scanner from a source.
    pub fn new(source: &'a str) -> Self {
        Scanner {
            source: source.as_bytes(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(mut self) -> Result<Vec<Token<'a>>, PiccoloError> {
        while self.next_token()?.kind != TokenKind::Eof {}
        Ok(self.tokens)
    }

    pub(crate) fn current(&self) -> &Token<'a> {
        &self.tokens[self.tokens.len() - 1]
    }

    fn slurp_whitespace(&mut self) {
        while self.peek() == b'#' || is_whitespace(self.peek()) {
            if self.peek() == b'#' {
                while self.peek() != b'\n' {
                    self.advance();
                }
            }
            while is_whitespace(self.peek()) {
                if self.advance() == b'\n' {
                    self.line += 1;
                }
            }
        }
    }

    pub(crate) fn next_token(&mut self) -> Result<&Token, PiccoloError> {
        self.slurp_whitespace();
        if self.is_at_end() {
            self.add_token(TokenKind::Eof);
            return Ok(self.current());
        }

        self.start = self.current;
        let tk = match self.advance() {
            b'[' => TokenKind::LeftBracket,
            b']' => TokenKind::RightBracket,
            b'(' => TokenKind::LeftParen,
            b')' => TokenKind::RightParen,
            b',' => TokenKind::Comma,
            b'-' => TokenKind::Minus,
            b'+' => TokenKind::Plus,
            b'*' => TokenKind::Multiply,
            b'/' => TokenKind::Divide,
            b'^' => TokenKind::BitwiseXor,
            b'%' => TokenKind::Modulo,

            b'&' => {
                if self.peek() == b'&' {
                    self.advance();
                    TokenKind::LogicalAnd
                } else {
                    TokenKind::BitwiseAnd
                }
            }

            b'|' => {
                if self.peek() == b'|' {
                    self.advance();
                    TokenKind::LogicalOr
                } else {
                    TokenKind::BitwiseOr
                }
            }

            b'.' => {
                if self.peek() == b'.' {
                    self.advance();
                    if self.peek() == b'.' {
                        self.advance();
                        TokenKind::InclusiveRange
                    } else {
                        TokenKind::ExclusiveRange
                    }
                } else {
                    TokenKind::Period
                }
            }

            b'!' => {
                if self.peek() == b'=' {
                    self.advance();
                    TokenKind::NotEqual
                } else {
                    TokenKind::Not
                }
            }

            b'=' => {
                if self.peek() == b'=' {
                    self.advance();
                    TokenKind::Equal
                } else {
                    TokenKind::Assign
                }
            }

            b'>' => {
                if self.peek() == b'=' {
                    self.advance();
                    TokenKind::GreaterEqual
                } else if self.peek() == b'>' {
                    self.advance();
                    TokenKind::ShiftRight
                } else {
                    TokenKind::Greater
                }
            }

            b'<' => {
                if self.peek() == b'=' {
                    self.advance();
                    TokenKind::LessEqual
                } else if self.peek() == b'<' {
                    self.advance();
                    TokenKind::ShiftLeft
                } else {
                    TokenKind::Less
                }
            }

            b'"' => self.string()?,

            c => {
                if is_digit(c) {
                    self.number()?
                } else if is_whitespace(c) {
                    panic!("found whitespace where there shouldn't be any");
                } else {
                    self.identifier_or_keyword()?
                }
            }
        };
        self.add_token(tk);
        Ok(self.current())
    }

    fn identifier_or_keyword(&mut self) -> Result<TokenKind, PiccoloError> {
        while !is_non_identifier(self.peek()) {
            self.advance();
        }

        if let Some(tk) = into_keyword(
            core::str::from_utf8(&self.source[self.start..self.current])
                .map_err(|_| PiccoloError::new(ErrorKind::InvalidUTF8).line(self.line))?,
        ) {
            Ok(tk)
        } else {
            Ok(TokenKind::Identifier)
        }
    }

    fn string(&mut self) -> Result<TokenKind, PiccoloError> {
        let line_start = self.line;
        while self.peek() != b'"' && !self.is_at_end() {
            if self.peek() == b'\n' {
                self.line += 1;
            }

            if self.peek() == b'\\' {
                self.advance();
                self.line += 1;
            }

            self.advance();
        }

        if self.is_at_end() {
            Err(PiccoloError::new(ErrorKind::UnterminatedString).line(line_start))
        } else {
            self.advance();
            Ok(TokenKind::String)
        }
    }

    fn number(&mut self) -> Result<TokenKind, PiccoloError> {
        while is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == b'.' {
            let range = self.lookahead(1) == b'.';

            while self.current != 0 && is_digit(self.peek()) {
                self.reverse();
            }

            if !range {
                return self.float();
            }
        }

        let value = core::str::from_utf8(&self.source[self.start..self.current])
            .map_err(|_| PiccoloError::new(ErrorKind::InvalidUTF8).line(self.line))?;
        if let Ok(i) = value.parse::<i64>() {
            Ok(TokenKind::Integer(i))
        } else {
            Err(PiccoloError::new(ErrorKind::InvalidNumberLiteral {
                literal: value.to_owned(),
            })
            .line(self.line))
        }
    }

    fn float(&mut self) -> Result<TokenKind, PiccoloError> {
        while is_digit(self.peek()) {
            self.advance();
        }
        if self.peek() == b'.' {
            self.advance();
            while is_digit(self.peek()) {
                self.advance();
            }
        }

        let value = core::str::from_utf8(&self.source[self.start..self.current])
            .map_err(|_| PiccoloError::new(ErrorKind::InvalidUTF8).line(self.line))?;
        if let Ok(f) = value.parse::<f64>() {
            Ok(TokenKind::Double(f))
        } else {
            Err(PiccoloError::new(ErrorKind::InvalidNumberLiteral {
                literal: value.to_owned(),
            })
            .line(self.line))
        }
    }

    fn add_token(&mut self, kind: TokenKind) {
        self.tokens.push(Token::new(
            kind,
            core::str::from_utf8(&self.source[self.start..self.current]).unwrap(),
            self.line,
        ));
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> u8 {
        self.current += 1;
        self.source[self.current - 1]
    }

    fn reverse(&mut self) -> u8 {
        self.current -= 1;
        self.source[self.current]
    }

    fn peek(&mut self) -> u8 {
        if self.is_at_end() {
            b'\0'
        } else {
            self.source[self.current]
        }
    }

    fn lookahead(&mut self, n: usize) -> u8 {
        if self.is_at_end() || self.current + n >= self.source.len() {
            b'\0'
        } else {
            self.source[self.current + n]
        }
    }
}

fn is_digit(c: u8) -> bool {
    b'0' <= c && c <= b'9'
}

pub(crate) fn is_whitespace(c: u8) -> bool {
    c == 0x09        // tab
        || c == 0x0A // line feed
        || c == 0x0B // line tab
        || c == 0x0C // form feed
        || c == 0x0D // carriage return
        || c == 0x20 // space
                     //  || c == 0x85 // next line      !! represented in utf-8 as C2 85
                     //  || c == 0xA0 // no-break space !! represented in utf-8 as C2 A0
}

fn is_non_identifier(c: u8) -> bool {
    is_whitespace(c)
        || c == 0x00
        || c == b'#'
        || c == b'['
        || c == b']'
        || c == b'('
        || c == b')'
        || c == b','
        || c == b'-'
        || c == b'+'
        || c == b'*'
        || c == b'/'
        || c == b'^'
        || c == b'%'
        || c == b'&'
        || c == b'|'
        || c == b'.'
        || c == b'!'
        || c == b':'
        || c == b'='
        || c == b'>'
        || c == b'<'
        || c == b'"'
}

#[cfg(feature = "fuzzer")]
impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenKind::Do => write!(f, "do"),
            TokenKind::End => write!(f, "end"),
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::While => write!(f, "while"),
            TokenKind::For => write!(f, "for"),
            TokenKind::In => write!(f, "in"),
            TokenKind::Data => write!(f, "data"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::Is => write!(f, "is"),
            TokenKind::Me => write!(f, "me"),
            TokenKind::New => write!(f, "new"),
            TokenKind::Err => write!(f, "err"),
            TokenKind::Retn => write!(f, "retn"),
            TokenKind::Assert => write!(f, "assert"),
            TokenKind::Nil => write!(f, "nil"),
            TokenKind::LeftBracket => write!(f, "["),
            TokenKind::RightBracket => write!(f, "]"),
            TokenKind::LeftParen => write!(f, "("),
            TokenKind::RightParen => write!(f, ")"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Period => write!(f, "."),
            TokenKind::ExclusiveRange => write!(f, ".."),
            TokenKind::InclusiveRange => write!(f, "..."),
            TokenKind::Assign => write!(f, "="),
            TokenKind::Not => write!(f, "!"),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Multiply => write!(f, "*"),
            TokenKind::Divide => write!(f, "/"),
            TokenKind::Modulo => write!(f, "%"),
            TokenKind::LogicalAnd => write!(f, "&&"),
            TokenKind::LogicalOr => write!(f, "||"),
            TokenKind::BitwiseAnd => write!(f, "&"),
            TokenKind::BitwiseOr => write!(f, "|"),
            TokenKind::BitwiseXor => write!(f, "^"),
            TokenKind::Equal => write!(f, "=="),
            TokenKind::NotEqual => write!(f, "!="),
            TokenKind::Less => write!(f, "<"),
            TokenKind::Greater => write!(f, ">"),
            TokenKind::LessEqual => write!(f, "<="),
            TokenKind::GreaterEqual => write!(f, ">="),
            TokenKind::ShiftLeft => write!(f, "<<"),
            TokenKind::ShiftRight => write!(f, ">>"),
            TokenKind::Identifier => write!(f, "ident"),
            TokenKind::String => write!(f, "\"str\""),
            TokenKind::True => write!(f, "true"),
            TokenKind::False => write!(f, "false"),
            TokenKind::Double(f64) => write!(f, "{}", f64),
            TokenKind::Integer(i64) => write!(f, "{}", i64),
            TokenKind::Eof => write!(f, ""),
        }
    }
}