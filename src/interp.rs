
extern crate backtrace;

use ::*;
use expr::ExprAccept;
use stmt::StmtAccept;
use value::{Value, is_equal, is_truthy};
use err::{ErrorKind, PiccoloError};
use token::TokenKind;

pub struct Interpreter {
    pub env: env::Scope,
}

impl Interpreter {
    #![allow(new_without_default_derive)]
    pub fn new() -> Self {
        Interpreter {
            env: stdlib::create_stdlib()
        }
    }

    pub fn interpret(&mut self, stmts: &[stmt::Stmt]) -> Result<Option<Value>, PiccoloError> {
        for stmt in stmts {
            if let Some(v) = self.execute(stmt)? {
                return Ok(Some(v))
            }
        }

        Ok(None)
    }

    pub fn interpret_with(&mut self, stmts: &[stmt::Stmt], mut env: &mut env::Scope) -> Result<Option<Value>, PiccoloError> {
        std::mem::swap(&mut self.env, &mut env);
        let res = self.interpret(stmts);
        std::mem::swap(&mut self.env, &mut env);
        res
    }

    pub fn execute_list(&mut self, stmts: &[stmt::Stmt]) -> Result<Option<Value>, PiccoloError> {
        self.env.push();
        let result = self.interpret(stmts);
        self.env.pop();
        result
    }

    pub fn execute(&mut self, stmt: &stmt::Stmt) -> Result<Option<Value>, PiccoloError> {
        stmt.accept(&mut *self)
    }

    pub fn evaluate(&mut self, expr: &expr::Expr) -> Result<Value, PiccoloError> {
        expr.accept(&mut *self)
    }

    fn error(&mut self, line: u64, kind: ErrorKind, why: &str) -> PiccoloError {
        PiccoloError::new(kind, why, line)
    }
}

impl expr::ExprVisitor for Interpreter {
    type Output = Result<Value, PiccoloError>;

    fn visit_binary(&mut self, e: &expr::Binary) -> Self::Output {
        let mut lhs = self.evaluate(&e.lhs)?;
        let mut rhs = self.evaluate(&e.rhs)?;

        Ok(match e.op.kind {
            TokenKind::Minus => match lhs {
                Value::Float(l) => match rhs {
                    Value::Float(r) => Value::Float(l - r),
                    Value::Integer(r) => Value::Float(l - r as f64),
                    _ => return Err(self.error(e.op.line, ErrorKind::MathError, &format!("Tried to subtract {:?} from {:?}", rhs, lhs)))

                },
                Value::Integer(l) => match rhs {
                    Value::Float(r) => Value::Float(l as f64 - r),
                    Value::Integer(r) => Value::Integer(l - r),
                    _ => return Err(self.error(e.op.line, ErrorKind::MathError, &format!("Tried to subtract {:?} from {:?}", rhs, lhs)))

                },
                _ => return Err(self.error(e.op.line, ErrorKind::MathError, &format!("Tried to subtract {:?} from {:?}", rhs, lhs)))
            },

            TokenKind::Plus => match lhs {
                Value::Float(l) => match rhs {
                    Value::Float(r) => Value::Float(l + r),
                    Value::Integer(r) => Value::Float(l + r as f64),
                    _ => return Err(self.error(e.op.line, ErrorKind::MathError, &format!("Tried to add {:?} to {:?}", lhs, rhs)))
                },
                Value::Integer(l) => match rhs {
                    Value::Float(r) => Value::Float(l as f64 + r),
                    Value::Integer(r) => Value::Integer(l + r),
                    _ => return Err(self.error(e.op.line, ErrorKind::MathError, &format!("Tried to add {:?} to {:?}", lhs, rhs)))
                },
                Value::String(ref mut l) => match rhs {
                    Value::String(ref mut r) => Value::String({l.push_str(r); l.clone()}),
                    r => return Err(self.error(e.op.line, ErrorKind::MathError, &format!("Tried to add {:?} to {:?}", l, r)))
                },
                _ => return Err(self.error(e.op.line, ErrorKind::MathError, &format!("Tried to add {:?} to {:?}", lhs, rhs)))
            },

            TokenKind::Divide => match lhs {
                Value::Float(l) => match rhs {
                    Value::Float(r) => Value::Float(l / r),
                    Value::Integer(r) => Value::Float(l / r as f64),
                    _ => return Err(self.error(e.op.line, ErrorKind::MathError, &format!("Tried to divide {:?} by {:?}", lhs, rhs)))
                },
                Value::Integer(l) => match rhs {
                    Value::Float(r) => Value::Float(l as f64 / r),
                    Value::Integer(r) => Value::Integer(l / r),
                    _ => return Err(self.error(e.op.line, ErrorKind::MathError, &format!("Tried to divide {:?} by {:?}", lhs, rhs)))
                },
                _ => return Err(self.error(e.op.line, ErrorKind::MathError, &format!("Tried to divide {:?} by {:?}", lhs, rhs)))
            },

            TokenKind::Star => match lhs {
                Value::Float(l) => match rhs {
                    Value::Float(r) => Value::Float(l * r),
                    Value::Integer(r) => Value::Float(l * r as f64),
                    _ => return Err(self.error(e.op.line, ErrorKind::MathError, &format!("Tried to multiply {:?} by {:?}", lhs, rhs)))
                },
                Value::Integer(l) => match rhs {
                    Value::Float(r) => Value::Float(l as f64 * r),
                    Value::Integer(r) => Value::Integer(l * r),
                    _ => return Err(self.error(e.op.line, ErrorKind::MathError, &format!("Tried to multiply {:?} by {:?}", lhs, rhs)))
                },
                _ => return Err(self.error(e.op.line, ErrorKind::MathError, &format!("Tried to multiply {:?} by {:?}", lhs, rhs)))
            },

            TokenKind::Mod => match lhs {
                Value::Float(l) => match rhs {
                    Value::Float(r) => Value::Float(l % r),
                    Value::Integer(r) => Value::Float(l % r as f64),
                    _ => return Err(self.error(e.op.line, ErrorKind::MathError, &format!("Tried to modulo {:?} by {:?}", lhs, rhs)))
                },
                Value::Integer(l) => match rhs {
                    Value::Float(r) => Value::Float(l as f64 % r),
                    Value::Integer(r) => Value::Integer(l % r),
                    _ => return Err(self.error(e.op.line, ErrorKind::MathError, &format!("Tried to modulo {:?} by {:?}", lhs, rhs)))
                },
                _ => return Err(self.error(e.op.line, ErrorKind::MathError, &format!("Tried to modulo {:?} by {:?}", lhs, rhs)))
            },

            TokenKind::GreaterThan => match lhs {
                Value::Float(l) => match rhs {
                    Value::Float(r) => Value::Bool(l > r),
                    Value::Integer(r) => Value::Bool(l > r as f64),
                    _ => return Err(self.error(e.op.line, ErrorKind::MathError, &format!("Tried to compare {:?} to {:?}", lhs, rhs)))
                },
                Value::Integer(l) => match rhs {
                    Value::Float(r) => Value::Bool(l as f64 > r),
                    Value::Integer(r) => Value::Bool(l > r),
                    _ => return Err(self.error(e.op.line, ErrorKind::MathError, &format!("Tried to compare {:?} to {:?}", lhs, rhs)))
                }
                _ => return Err(self.error(e.op.line, ErrorKind::MathError, &format!("Tried to compare {:?} to {:?}", lhs, rhs)))
            },

            TokenKind::LessThan => match lhs {
                Value::Float(l) => match rhs {
                    Value::Float(r) => Value::Bool(l < r),
                    Value::Integer(r) => Value::Bool(l < r as f64),
                    _ => return Err(self.error(e.op.line, ErrorKind::MathError, &format!("Tried to compare {:?} to {:?}", lhs, rhs)))
                },
                Value::Integer(l) => match rhs {
                    Value::Float(r) => Value::Bool((l as f64) < r),
                    Value::Integer(r) => Value::Bool(l < r),
                    _ => return Err(self.error(e.op.line, ErrorKind::MathError, &format!("Tried to compare {:?} to {:?}", lhs, rhs)))
                }
                _ => return Err(self.error(e.op.line, ErrorKind::MathError, &format!("Tried to compare {:?} to {:?}", lhs, rhs)))
            },

            TokenKind::GreaterThanEquals => match lhs {
                Value::Float(l) => match rhs {
                    Value::Float(r) => Value::Bool(l >= r),
                    Value::Integer(r) => Value::Bool(l >= r as f64),
                    _ => return Err(self.error(e.op.line, ErrorKind::MathError, &format!("Tried to compare {:?} to {:?}", lhs, rhs)))
                },
                Value::Integer(l) => match rhs {
                    Value::Float(r) => Value::Bool(l as f64 >= r),
                    Value::Integer(r) => Value::Bool(l >= r),
                    _ => return Err(self.error(e.op.line, ErrorKind::MathError, &format!("Tried to compare {:?} to {:?}", lhs, rhs)))
                }
                _ => return Err(self.error(e.op.line, ErrorKind::MathError, &format!("Tried to compare {:?} to {:?}", lhs, rhs)))
            },

            TokenKind::LessThanEquals => match lhs {
                Value::Float(l) => match rhs {
                    Value::Float(r) => Value::Bool(l <= r),
                    Value::Integer(r) => Value::Bool(l <= r as f64),
                    _ => return Err(self.error(e.op.line, ErrorKind::MathError, &format!("Tried to compare {:?} to {:?}", lhs, rhs)))
                },
                Value::Integer(l) => match rhs {
                    Value::Float(r) => Value::Bool(l as f64 <= r),
                    Value::Integer(r) => Value::Bool(l <= r),
                    _ => return Err(self.error(e.op.line, ErrorKind::MathError, &format!("Tried to compare {:?} to {:?}", lhs, rhs)))
                }
                _ => return Err(self.error(e.op.line, ErrorKind::MathError, &format!("Tried to compare {:?} to {:?}", lhs, rhs)))
            },

            TokenKind::Equals => Value::Bool(is_equal(&lhs, &rhs)),
            TokenKind::NotEquals => Value::Bool(!is_equal(&lhs, &rhs)),

            TokenKind::ERange => match lhs {
                Value::Integer(l) => match rhs {
                    Value::Integer(r) => {
                        Value::Array((l..r).map(|n| n.into()).collect())
                    },
                    _ => return Err(self.error(e.op.line, ErrorKind::MathError, &format!("Tried to create range {:?}..{:?}", lhs, rhs)))
                },
                _ => return Err(self.error(e.op.line, ErrorKind::MathError, &format!("Tried to create range {:?}..{:?}", lhs, rhs)))
            },

            TokenKind::IRange => match lhs {
                Value::Integer(l) => match rhs {
                    Value::Integer(r) => {
                        Value::Array((l..r + 1).map(|n| n.into()).collect())
                    },
                    _ => return Err(self.error(e.op.line, ErrorKind::MathError, &format!("Tried to create range {:?}...{:?}", lhs, rhs)))
                },
                _ => return Err(self.error(e.op.line, ErrorKind::MathError, &format!("Tried to create range {:?}...{:?}", lhs, rhs)))
            },

            v => panic!("unreachable: {:?} {}", v, e.op.line)
        })
    }

    fn visit_unary(&mut self, e: &expr::Unary) -> Self::Output {
        let rhs = self.evaluate(&e.rhs)?;

        match e.op.kind {
            TokenKind::Minus => match rhs {
                Value::Integer(ref n) => Ok((-n).into()),
                Value::Float(ref n) => Ok((-n).into()),
                v => Err(self.error(e.op.line, ErrorKind::MathError, &format!("Tried to negate non-bool/number {:?}", v)))
            },
            TokenKind::Not => {
                let b: bool = is_truthy(&rhs);
                Ok(Value::Bool(!b))
            },
            _ => Err(self.error(e.op.line, ErrorKind::MathError, &format!("Not a unary operator: \"{}\"", e.op.lexeme)))
        }
    }

    fn visit_paren(&mut self, e: &expr::Paren) -> Self::Output {
        self.evaluate(&e.0)
    }

    fn visit_literal(&mut self, e: &expr::Literal) -> Self::Output {
        match e {
            &expr::Literal::Array(expr::Array { ref inner, .. }) => {
                let mut new = Vec::new();
                for item in inner.iter() {
                    let new_item = self.evaluate(item)?;
                    new.push(new_item);
                }
                Ok(Value::Array(new))
            },
            l => Ok(std::mem::replace(&mut l.clone(), expr::Literal::Nil).into())
        }
    }

    fn visit_variable(&mut self, e: &expr::Variable) -> Self::Output {
        if let Some(v) = self.env.get(&e.0.lexeme) {
            Ok(v)
        } else {
            Err(self.error(e.0.line, ErrorKind::UndefinedVariable, &format!("{} is undefined", e.0.lexeme)))
        }
    }

    fn visit_assign(&mut self, e: &expr::Assignment) -> Self::Output {
        let value = self.evaluate(&e.value)?;
        self.env.set(&e.name.lexeme, value.clone());
        Ok(value)
    }

    fn visit_logical(&mut self, e: &expr::Logical) -> Self::Output {
        let lhs = self.evaluate(&e.lhs)?;

        if e.op.kind == token::TokenKind::Or {
            if is_truthy(&lhs) {
                return Ok(lhs)
            }
        } else if !is_truthy(&lhs) {
            return Ok(lhs)
        }

        self.evaluate(&e.rhs)
    }

    fn visit_call(&mut self, e: &expr::Call) -> Self::Output {
        let callee = self.evaluate(&e.callee)?;

        let mut func = match callee {
            Value::Func(f) => f,
            v => {
                return Err(self.error(e.paren.line, ErrorKind::NonFunction, &format!("Attempt to call non-function {:?}", v)));
            }
        };

        let args: Result<Vec<Value>, PiccoloError> = e.args.iter().map(|arg| self.evaluate(arg)).collect();
        let args = args?;

        if !func.arity.compatible(e.arity) {
            return Err(self.error(e.paren.line, ErrorKind::IncorrectArity, &format!("Expected {} args, got {}", func.arity.to_number(), args.len())));
        }

        let result = func.call(&mut *self, &args);

        result
    }

    fn visit_new(&mut self, e: &expr::New) -> Self::Output {
        use std::collections::HashMap;

        if let Some(Value::Data(data)) = self.env.get(&e.name.lexeme) {
            let mut fields: HashMap<String, data::Field> = HashMap::new();
            for (name, value) in &data.fields {
                fields.insert(name.clone(), value.clone());
            }
            for &(ref name, ref value) in &e.args {
                let f = fields.get(name).cloned();
                if let Some(ref field) = f {
                    if field.public {
                        fields.insert(name.clone(), data::Field {
                            normal: true,
                            public: true,
                            value: self.evaluate(value)?,
                        });
                    } else {
                        return Err(self.error(e.name.line, ErrorKind::NoSuchField, &format!("Field {} is private", name)))
                    }
                } else {
                    return Err(self.error(e.name.line, ErrorKind::NoSuchField, &format!("Field {} does not exist", name)))
                }
            }
            Ok(Value::Instance(data::Instance::new(&data, fields)))
        } else {
            Err(self.error(e.name.line, ErrorKind::NonData, "Tried to create data from non-data"))
        }
    }

    fn visit_get(&mut self, e: &expr::Get) -> Self::Output {
        let me = match *e.object {
            expr::Expr::Variable(expr::Variable(ref v)) => v.lexeme == "me",
            _ => false
        };
        let value = self.evaluate(&*e.object)?;
        if let Value::Instance(ref inst) = value {
            if let Some(field) = inst.get(&e.name.lexeme, me) {
                Ok(field)
            } else {
                Err(self.error(e.name.line, ErrorKind::NoSuchField, &format!("No field named {}", e.name.lexeme)))
            }
        } else {
            Err(self.error(e.name.line, ErrorKind::NonInstance, "Non-instance does not have fields"))
        }
    }

    fn visit_set(&mut self, e: &expr::Set) -> Self::Output {
        match &*e.object {
            &expr::Expr::Index(ref expr) => {
                let i = self.evaluate(&*expr.i)?;
                match i {
                    Value::Integer(idx) => {
                        panic!("{:?}", expr);
                    },
                    idx => Err(self.error(expr.rb.line, ErrorKind::IndexError, &format!("Cannot index with non-integer {:?}", idx)))
                }
            },
            _ => {
                let value = self.evaluate(&*e.object)?;
                if let Value::Instance(ref instance) = value {
                    let value = self.evaluate(&*e.value)?;
                    instance.set(&e.name.lexeme, value.clone()).map(|_| value).map_err(|_| self.error(e.name.line, ErrorKind::NoSuchField, &format!("No such field named {}", e.name.lexeme)))
                    //Ok(value)
                } else {
                    Err(self.error(e.name.line, ErrorKind::NonInstance, "Non-instance does not have fields"))
                }
            }
        }
        //let value = self.evaluate(&*e.object)?;
        //match value {
        //    Value::Instace(ref instance) => {
        //        let value = self.evaluate(&*e.value)?;
        //        instance.set(&e.name.lexeme, value.clone());
        //        Ok(value)
        //    },
        //    Value::Array(ref mut r) => {
        //        println
        //    }
        //}
        //println!("{:?}", e);
        //if let Value::Instance(ref instance) = value {
        //    let value = self.evaluate(&*e.value)?;
        //    instance.set(&e.name.lexeme, value.clone());
        //    Ok(value)
        //} else {
        //    Err(self.error(e.name.line, ErrorKind::NonInstance, "Non-instance does not have fields"))
        //}
    }

    fn visit_index(&mut self, e: &expr::Index) -> Self::Output {
        let object = self.evaluate(&*e.object)?;
        let i = self.evaluate(&*e.i)?;
        match i {
            Value::Integer(i) => match object {
                Value::Array(ref a) => {
                    if (i as usize) < a.len() {
                        Ok(a[i as usize].clone())
                    } else {
                        Err(self.error(e.rb.line, ErrorKind::IndexError, &format!("Index out-of-bounds: {}", i)))
                    }
                },
                v => Err(self.error(e.rb.line, ErrorKind::IndexError, &format!("Cannot index non-array {:?}", v)))
            },
            i => Err(self.error(e.rb.line, ErrorKind::IndexError, &format!("Cannot index with non-integer {:?}", i)))
        }
    }
}

impl stmt::StmtVisitor for Interpreter {
    type Output = Result<Option<Value>, PiccoloError>;

    fn visit_expr(&mut self, s: &stmt::StmtExpr) -> Self::Output {
        s.0.accept(&mut *self)?;
        Ok(None)
    }

    fn visit_assignment(&mut self, s: &stmt::Assignment) -> Self::Output {
        let value = self.evaluate(&s.value)?;
        self.env.set(&s.name.lexeme, value);
        Ok(None)
    }

    fn visit_block(&mut self, s: &stmt::Block) -> Self::Output {
        self.execute_list(&s.0)
    }

    fn visit_if(&mut self, s: &stmt::If) -> Self::Output {
        let cond = self.evaluate(&s.cond)?;

        let result = if is_truthy(&cond) {
            self.execute_list(&s.then)?
        } else if let Some(ref else_) = s.else_ {
            self.execute_list(else_)?
        } else {
            None
        };

        Ok(result)
    }

    fn visit_while(&mut self, s: &stmt::While) -> Self::Output {
        let mut cond = self.evaluate(&s.cond)?;
        while is_truthy(&cond) {
            if let Some(v) = self.execute_list(&s.body)? {
                return Ok(Some(v))
            }
            cond = self.evaluate(&s.cond)?;
        }
        Ok(None)
    }

    fn visit_for(&mut self, s: &stmt::For) -> Self::Output {
        let iter = self.evaluate(&s.iter)?;
        self.env.push();
        match iter {
            Value::Array(ref a) => {
                for item in a {
                    self.env.set(&s.name.lexeme, item.clone());
                    if let Some(r) = self.interpret(&s.body)? {
                        self.env.pop();
                        return Ok(Some(r))
                    }
                }
            }
            _ => return Err(self.error(s.name.line, ErrorKind::NonIterator, &format!("Cannot be iterated over: {:?}", iter))),
        }
        self.env.pop();
        Ok(None)
    }

    fn visit_func(&mut self, s: &stmt::Func) -> Self::Output {
        let func = Value::Func(func::Func::new(s.arity, s.clone()));
        self.env.set(&s.name.lexeme, func);
        Ok(None)
    }

    fn visit_retn(&mut self, s: &stmt::Retn) -> Self::Output {
        let value = if let Some(ref e) = s.value {
            self.evaluate(e)?
        } else {
            Value::Nil
        };

        Ok(Some(value))
    }

    fn visit_data(&mut self, s: &stmt::Data) -> Self::Output {
        let mut fields = std::collections::HashMap::new();
        let mut methods = std::collections::HashMap::new();
        for &(public, ref name, ref value) in &s.fields {
            fields.insert(name.lexeme.clone(), data::Field {
                normal: public,
                public, value: self.evaluate(value)?,
            });
        }

        for func in &s.methods {
            methods.insert(func.name.lexeme.clone(), data::Field {
                normal: true, // TODO
                public: true, // TODO
                value: Value::Func(func::Func::new_method(func.arity, func.clone())),
            });
        }

        let data = data::Data::new(&s.name.lexeme, fields, methods);
        self.env.set(&s.name.lexeme, Value::Data(data));
        Ok(None)
    }
}

