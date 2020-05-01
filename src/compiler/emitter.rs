//! Contains `Emitter`, which converts an AST into a [`Chunk`].
//!
//! [`Chunk`]: ../runtime/chunk/struct.Chunk.html

use crate::runtime::op::Opcode;
use crate::{Chunk, Constant, ErrorKind, PiccoloError, Token, TokenKind};

use super::ast::{Arity, Expr, ExprAccept, ExprVisitor, Stmt, StmtAccept, StmtVisitor};

use std::collections::HashMap;

/// Struct for emitting Piccolo virtual machine bytecode.
///
/// Implements [`StmtVisitor`] and [`ExprVisitor`] to walk the AST, compiling
/// into a [`Chunk`].
///
/// [`ExprVisitor`]: ../ast/trait.ExprVisitor.html
/// [`StmtVisitor`]: ../ast/trait.StmtVisitor.html
/// [`Chunk`]: ../runtime/chunk/struct.Chunk.html
pub struct Emitter {
    chunk: Chunk,
    strings: HashMap<String, u16>,
    identifiers: HashMap<String, u16>,
    scope_depth: u16,
    locals: Vec<(String, u16)>,
}

impl Emitter {
    /// Create a new bytecode emitter.
    pub fn new(chunk: Chunk) -> Self {
        Emitter {
            chunk,
            strings: HashMap::new(),
            identifiers: HashMap::new(),
            scope_depth: 0,
            locals: Vec::new(),
        }
    }

    /// Get the chunk of the emitter.
    pub fn chunk(&self) -> &Chunk {
        &self.chunk
    }

    /// Compile an AST into a chunk. Moves the emitter's chunk out of itself,
    /// replacing it with [`Chunk::default`].
    ///
    /// [`Chunk::default`]: ../../runtime/chunk/struct.Chunk.html
    pub fn compile(&mut self, stmts: &[Stmt]) -> Result<Chunk, Vec<PiccoloError>> {
        let mut errs = Vec::new();
        for stmt in stmts {
            trace!("stmt {}", super::ast::AstPrinter::print_stmt(stmt));

            match stmt.accept(self) {
                Ok(_) => {}
                Err(e) => errs.push(e),
            }
        }

        if errs.is_empty() {
            Ok(std::mem::take(&mut self.chunk))
        } else {
            Err(errs)
        }
    }

    fn get_local_slot(&self, name: &str) -> Option<u16> {
        trace!("get local slot for '{}'", name);

        for (i, (k, _)) in self.locals.iter().enumerate().rev() {
            if k == name {
                return Some(i as u16);
            }
        }
        None
    }

    fn get_local_depth(&self, name: &str) -> Option<u16> {
        trace!("get local depth for '{}'", name);

        for (k, v) in self.locals.iter().rev() {
            if k == name {
                return Some(*v);
            }
        }
        None
    }

    fn make_ident(&mut self, name: &str) -> u16 {
        trace!("make ident '{}'", name);

        if self.identifiers.contains_key(name) {
            self.identifiers[name]
        } else {
            let idx = self.chunk.make_constant(Constant::String(name.to_owned()));
            self.identifiers.insert(name.to_owned(), idx);
            idx
        }
    }

    fn get_ident(&self, name: &Token) -> Result<u16, PiccoloError> {
        trace!("get ident '{}'", name.lexeme);

        if self.identifiers.contains_key(name.lexeme) {
            Ok(self.identifiers[name.lexeme])
        } else {
            Err(PiccoloError::new(ErrorKind::UndefinedVariable {
                name: name.lexeme.to_owned(),
            })
            .line(name.line))
        }
    }
}

impl ExprVisitor for Emitter {
    type Output = Result<(), PiccoloError>;

    fn visit_atom(&mut self, token: &Token) -> Self::Output {
        let i = if token.kind == TokenKind::String && self.strings.contains_key(token.lexeme) {
            trace!("has string {}", token.lexeme);

            *self.strings.get(token.lexeme).unwrap()
        } else if token.kind == TokenKind::String {
            let i = self.chunk.make_constant(Constant::try_from(*token)?);
            self.strings.insert(token.lexeme.to_string(), i);
            i
        } else if token.kind == TokenKind::Nil {
            self.chunk.write_u8(Opcode::Nil, token.line);
            return Ok(());
        } else if token.kind == TokenKind::True {
            self.chunk.write_u8(Opcode::True, token.line);
            return Ok(());
        } else if token.kind == TokenKind::False {
            self.chunk.write_u8(Opcode::False, token.line);
            return Ok(());
        } else {
            self.chunk
                .make_constant(Constant::try_from(*token).unwrap())
        };

        self.chunk.write_arg_u16(Opcode::Constant, i, token.line);

        Ok(())
    }

    fn visit_paren(&mut self, value: &Expr) -> Self::Output {
        value.accept(self)
    }

    fn visit_variable(&mut self, name: &Token) -> Self::Output {
        match self.get_local_slot(name.lexeme) {
            Some(idx) => self.chunk.write_arg_u16(Opcode::GetLocal, idx, name.line),
            None => {
                let i = self.get_ident(name)?;
                self.chunk.write_arg_u16(Opcode::GetGlobal, i, name.line);
            }
        }
        Ok(())
    }

    fn visit_unary(&mut self, op: &Token, rhs: &Expr) -> Self::Output {
        rhs.accept(self)?;
        match op.kind {
            TokenKind::Not => self.chunk.write_u8(Opcode::Not, op.line),
            TokenKind::Minus => self.chunk.write_u8(Opcode::Negate, op.line),
            _ => unreachable!("unrecognized unary op {:?}", op),
        }
        Ok(())
    }

    fn visit_binary(&mut self, lhs: &Expr, op: &Token, rhs: &Expr) -> Self::Output {
        lhs.accept(self)?;
        rhs.accept(self)?;

        match op.kind {
            TokenKind::Plus => self.chunk.write_u8(Opcode::Add, op.line),
            TokenKind::Minus => self.chunk.write_u8(Opcode::Subtract, op.line),
            TokenKind::Divide => self.chunk.write_u8(Opcode::Divide, op.line),
            TokenKind::Multiply => self.chunk.write_u8(Opcode::Multiply, op.line),
            TokenKind::Equal => self.chunk.write_u8(Opcode::Equal, op.line),
            TokenKind::NotEqual => {
                self.chunk.write_u8(Opcode::Equal, op.line);
                self.chunk.write_u8(Opcode::Not, op.line);
            }
            TokenKind::Greater => self.chunk.write_u8(Opcode::Greater, op.line),
            TokenKind::GreaterEqual => self.chunk.write_u8(Opcode::GreaterEqual, op.line),
            TokenKind::Less => self.chunk.write_u8(Opcode::Less, op.line),
            TokenKind::LessEqual => self.chunk.write_u8(Opcode::LessEqual, op.line),
            TokenKind::Modulo => self.chunk.write_u8(Opcode::Modulo, op.line),
            _ => unreachable!("unrecognized binary op {:?}", op),
        }

        Ok(())
    }

    fn visit_logical(&mut self, lhs: &Expr, _op: &Token, rhs: &Expr) -> Self::Output {
        lhs.accept(self)?;
        rhs.accept(self)
    }

    fn visit_call(
        &mut self,
        _callee: &Expr,
        _paren: &Token,
        _arity: Arity,
        _args: &[Expr],
    ) -> Self::Output {
        todo!("visit_call")
    }

    fn visit_new(&mut self, _name: &Token, _args: &[(Token, Box<Expr>)]) -> Self::Output {
        todo!("visit_new")
    }

    fn visit_get(&mut self, _object: &Expr, _name: &Token) -> Self::Output {
        todo!("visit_get")
    }

    fn visit_set(&mut self, _object: &Expr, _name: &Token, _value: &Expr) -> Self::Output {
        todo!("visit_set")
    }

    fn visit_index(&mut self, _rb: &Token, _object: &Expr, _idx: &Expr) -> Self::Output {
        todo!("visit_index")
    }

    fn visit_func(
        &mut self,
        _name: &Token,
        _args: &[Token],
        _arity: Arity,
        _body: &[Stmt],
        _method: bool,
    ) -> Self::Output {
        todo!("visit_func")
    }
}

impl StmtVisitor for Emitter {
    type Output = Result<(), PiccoloError>;

    fn visit_expr(&mut self, expr: &Expr) -> Self::Output {
        expr.accept(self)?;
        self.chunk.write_u8(Opcode::Pop, 1); // TODO: line
        Ok(())
    }

    fn visit_block(&mut self, end: &Token, body: &[Stmt]) -> Self::Output {
        self.scope_depth += 1;
        for stmt in body {
            stmt.accept(self)?;
        }
        self.scope_depth -= 1;
        while !self.locals.is_empty() && self.locals[self.locals.len() - 1].1 > self.scope_depth {
            self.chunk.write_u8(Opcode::Pop, end.line);
            self.locals.pop().unwrap();
        }
        Ok(())
    }

    fn visit_assignment(&mut self, name: &Token, op: &Token, value: &Expr) -> Self::Output {
        value.accept(self)?;
        if self.scope_depth > 0 {
            // inside of a block, declaration creates local variables
            if op.kind == TokenKind::Assign {
                if let Some(idx) = self.get_local_slot(name.lexeme) {
                    // local variable exists, reassign it
                    self.chunk.write_arg_u16(Opcode::SetLocal, idx, op.line);
                } else {
                    // reassign global variable, checking for existence at runtime
                    let i = self.get_ident(name)?;
                    self.chunk.write_arg_u16(Opcode::SetGlobal, i, name.line);
                }
            } else if op.kind == TokenKind::Declare {
                // if there exists some local with this name
                if let Some(idx) = self.get_local_depth(name.lexeme) {
                    if idx != self.scope_depth {
                        // create a new local if we're in a different scope
                        self.locals.push((name.lexeme.to_owned(), self.scope_depth));
                    } else {
                        // error if we're in the same scope
                        return Err(PiccoloError::new(ErrorKind::SyntaxError)
                            .line(name.line)
                            .msg_string(format!(
                                "cannot shadow local variable '{}'",
                                self.locals[idx as usize - 1].0
                            )));
                    }
                } else {
                    // create a new local with this name
                    self.locals.push((name.lexeme.to_owned(), self.scope_depth));
                }
            }
        } else if op.kind == TokenKind::Assign {
            let idx = self.get_ident(name)?;
            self.chunk.write_arg_u16(Opcode::SetGlobal, idx, name.line);
        } else if op.kind == TokenKind::Declare {
            let idx = self.make_ident(name.lexeme);
            self.chunk
                .write_arg_u16(Opcode::DeclareGlobal, idx, name.line);
        }
        Ok(())
    }

    fn visit_if(
        &mut self,
        cond: &Expr,
        do_: &Token,
        then: &[Stmt],
        else_: Option<&Vec<Stmt>>,
        end: &Token,
    ) -> Self::Output {
        // compile the condition
        cond.accept(self)?;

        // jump over the do block if the condition is false
        let cond_false = self.chunk.write_jump(Opcode::JumpFalse, do_.line);

        // pop the condition, it's still on the stack
        self.chunk.write_u8(Opcode::Pop, do_.line);
        // compile the do block
        self.visit_block(end, then)?;
        // if there's an else block, jump over it
        let cond_true = self.chunk.write_jump(Opcode::Jump, do_.line); // todo: wrong line number

        // jump here if the condition is false
        self.chunk.patch_jump(cond_false);

        if let Some(block) = else_ {
            // compile the else block
            self.visit_block(end, block)?;
        }

        // jump here if the condition is true
        self.chunk.patch_jump(cond_true);

        Ok(())
    }

    fn visit_while(&mut self, _cond: &Expr, _body: &[Stmt]) -> Self::Output {
        todo!("visit_while")
    }

    fn visit_for(&mut self, _name: &Token, _iter: &Expr, _body: &[Stmt]) -> Self::Output {
        todo!("visit_for")
    }

    fn visit_func(
        &mut self,
        _name: &Token,
        _args: &[Token],
        _arity: Arity,
        _body: &[Stmt],
        _method: bool,
    ) -> Self::Output {
        todo!("visit_func")
    }

    fn visit_retn(&mut self, keyword: &Token, value: Option<&Expr>) -> Self::Output {
        if let Some(expr) = value {
            expr.accept(self)?;
        }
        self.chunk.write_u8(Opcode::Return, keyword.line);
        Ok(())
    }

    fn visit_assert(&mut self, keyword: &Token, value: &Expr) -> Self::Output {
        value.accept(self)?;
        self.chunk.write_u8(Opcode::Assert, keyword.line);
        Ok(())
    }

    fn visit_data(
        &mut self,
        _name: &Token,
        _methods: &[Stmt],
        _fields: &[(Token, Expr)],
    ) -> Self::Output {
        todo!("visit_data")
    }
}
