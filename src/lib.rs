//! # Piccolo
//!
//! Piccolo is a small, light, high-pitched scripting language (eventually) intended
//! for embedding in Rust projects.

pub extern crate downcast_rs;
pub extern crate slotmap;

pub mod ast;
pub mod compiler;
pub mod error;
pub mod runtime;

pub use compiler::emitter::compile;
pub use compiler::scanner::{Scanner, Token, TokenKind};
pub use error::PiccoloError;
pub use runtime::{chunk::Chunk, value::Value, vm::Machine};

#[cfg(feature = "pc-debug")]
pub use compiler::scanner::print_tokens;

/// Interprets a Piccolo source and returns its result.
///
/// # Examples
///
/// ```rust
/// # fn main() -> Result<(), Vec<piccolo::PiccoloError>> {
/// let result = piccolo::interpret("1 + 2")?;
/// assert_eq!(3, result.into::<i64>());
/// # Ok(())
/// # }
/// ```
pub fn interpret(src: &str) -> Result<Value, Vec<error::PiccoloError>> {
    match Machine::new(compile(
        Chunk::default(),
        &Scanner::new(src).scan_tokens().map_err(|e| vec![e])?,
    )?)
    .interpret()
    {
        Ok(v) => Ok(v),
        Err(e) => Err(vec![e]),
    }
}

pub fn do_file(
    file: &std::path::Path,
) -> Result<Result<Value, Vec<error::PiccoloError>>, std::io::Error> {
    let contents = std::fs::read_to_string(file)?;
    Ok(interpret(&contents).map_err(|v| {
        v.into_iter()
            .map(|e| e.file(file.to_str().unwrap().to_owned()))
            .collect()
    }))
}

pub(crate) fn encode_bytes(low: u8, high: u8) -> u16 {
    ((high as u16) << 8) | (low as u16)
}

pub(crate) fn decode_bytes(bytes: u16) -> (u8, u8) {
    let high = (bytes >> 8) as u8;
    let low = (bytes & 0xff) as u8;
    (low, high)
}

#[cfg(feature = "fuzzer")]
pub mod fuzzer {
    extern crate rand;

    use crate::compiler::scanner::TokenKind;
    use crate::{Chunk, Machine, Scanner};
    use rand::distributions::{Distribution, Standard};
    use rand::Rng;

    /// Run `n` tests of random tokens.
    pub fn fuzz(n: usize, min_len: usize, max_len: usize) -> Option<Vec<usize>> {
        let mut ok = None;
        let start = std::time::Instant::now();
        let mut avg = 0.0;
        for n in 1..=n {
            let s = std::time::Instant::now();
            if let Some(_) = run(n, min_len, max_len) {
                if ok.is_none() {
                    ok = Some(vec![n]);
                } else {
                    ok.as_mut().unwrap().push(n);
                }
            }
            avg += (std::time::Instant::now() - s).as_secs_f64();
        }
        println!(
            "{} runs, in {:.8} sec ({:.8} avg per run)",
            n,
            (std::time::Instant::now() - start).as_secs_f64(),
            avg / n as f64
        );
        ok
    }

    // occasionally creates valid programs
    fn run(n: usize, min_len: usize, max_len: usize) -> Option<()> {
        let mut src = String::new();
        let mut r = rand::thread_rng();
        let lines = r.gen_range(min_len, max_len);
        for _ in 1..lines {
            let tk: TokenKind = r.gen();
            src.push_str(&format!("{} ", tk).to_lowercase());
        }

        let tokens = Scanner::new(&src).scan_tokens().ok()?;
        if let Ok(chunk) = crate::compile(Chunk::default(), &tokens) {
            println!("----- run {} compiles -----", n);
            crate::scanner::print_tokens(&tokens);
            chunk.disassemble("");
            Machine::new(chunk).interpret().ok().map(|_| {
                println!("----- run {} executes -----", n);
            })
        } else {
            None
        }
    }

    impl Distribution<TokenKind> for Standard {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> TokenKind {
            match rng.gen_range(0, 50) {
                // 0 => TokenKind::Do,
                // 1 => TokenKind::End,
                // 2 => TokenKind::Fn,
                // 3 => TokenKind::If,
                // 4 => TokenKind::Else,
                // 5 => TokenKind::While,
                // 6 => TokenKind::For,
                // 7 => TokenKind::In,
                // 8 => TokenKind::Data,
                9 => TokenKind::Let,
                // 10 => TokenKind::Is,
                // 11 => TokenKind::Me,
                // 12 => TokenKind::New,
                // 13 => TokenKind::Err,
                14 => TokenKind::Retn,
                15 => TokenKind::Nil,
                // 16 => TokenKind::LeftBracket,
                // 17 => TokenKind::RightBracket,
                18 => TokenKind::LeftParen,
                19 => TokenKind::RightParen,
                // 20 => TokenKind::Comma,
                // 21 => TokenKind::Period,
                // 22 => TokenKind::ExclusiveRange,
                // 23 => TokenKind::InclusiveRange,
                24 => TokenKind::Assign,
                25 => TokenKind::Not,
                26 => TokenKind::Plus,
                27 => TokenKind::Minus,
                28 => TokenKind::Multiply,
                29 => TokenKind::Divide,
                30 => TokenKind::Modulo,
                // 31 => TokenKind::LogicalAnd,
                // 32 => TokenKind::LogicalOr,
                // 33 => TokenKind::BitwiseAnd,
                // 34 => TokenKind::BitwiseOr,
                // 35 => TokenKind::BitwiseXor,
                36 => TokenKind::Equal,
                37 => TokenKind::NotEqual,
                38 => TokenKind::Less,
                39 => TokenKind::Greater,
                40 => TokenKind::LessEqual,
                41 => TokenKind::GreaterEqual,
                // 42 => TokenKind::ShiftLeft,
                // 43 => TokenKind::ShiftRight,
                44 => TokenKind::Identifier,
                45 => TokenKind::String,
                46 => TokenKind::True,
                47 => TokenKind::False,
                48 => TokenKind::Double(0.0),
                49 => TokenKind::Integer(1),
                _ => TokenKind::Nil,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::compiler::emitter::Precedence;
    use crate::runtime::op::Opcode;
    use crate::{Chunk, Machine, Scanner};

    #[test]
    fn get_line_from_index() {
        let mut c = Chunk::default();
        c.write(Opcode::Return, 1); // 0
        c.write(Opcode::Return, 1); // 1
        c.write(Opcode::Return, 1); // 2
        c.write(Opcode::Return, 1); // 3
        c.write(Opcode::Return, 1); // 4
        c.write(Opcode::Return, 1); // 5
        c.write(Opcode::Return, 2); // 6
        c.write(Opcode::Return, 2); // 7
        c.write(Opcode::Return, 2); // 8
        c.write(Opcode::Return, 2); // 9
        c.write(Opcode::Return, 2); // 10
        c.write(Opcode::Return, 3); // 11
        c.write(Opcode::Return, 3); // 12
        c.write(Opcode::Return, 3); // 13
        c.write(Opcode::Return, 3); // 14
        c.write(Opcode::Return, 4); // 15
        c.write(Opcode::Return, 4); // 16
        c.write(Opcode::Return, 4); // 17
        c.write(Opcode::Return, 4); // 18
        c.write(Opcode::Return, 5); // 19

        assert_eq!(c.get_line_from_index(0), 1);
        assert_eq!(c.get_line_from_index(5), 1);
        assert_eq!(c.get_line_from_index(6), 2);
        assert_eq!(c.get_line_from_index(10), 2);
        assert_eq!(c.get_line_from_index(11), 3);
        assert_eq!(c.get_line_from_index(14), 3);
    }

    #[test]
    fn encode_decode() {
        let bytes: u16 = 0xbead;
        let (low, high) = crate::decode_bytes(bytes);
        assert_eq!(high, 0xbe);
        assert_eq!(low, 0xad);

        let bytes2 = crate::encode_bytes(low, high);
        assert_eq!(bytes, bytes2);
    }

    #[test]
    fn very_long() {
        let len = 2048;
        let mut source = String::new();
        for i in 0..len {
            source.push_str(&format!("let a{:04x}=\"{}\"\n", i, i));
        }
        for i in 0..len {
            source.push_str(&format!("let b{:04x}=a{:04x}\n", i, i));
        }
        for i in 0..len {
            source.push_str(&format!("a{:04x}=b{:04x}\n", len - i - 1, i));
        }
        for i in 0..len {
            source.push_str(&format!("retn a{:04x}\n", i));
        }
        let chunk = crate::compile(
            Chunk::default(),
            &Scanner::new(&source).scan_tokens().unwrap(),
        )
        .unwrap();
        Machine::new(chunk).interpret().unwrap();
    }

    #[test]
    fn precedence_ord() {
        assert!(Precedence::And > Precedence::Or);
    }
}
