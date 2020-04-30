//! Contains `Machine`, the Piccolo bytecode interpreter.

use crate::runtime::memory::Heap;
use crate::{Chunk, Constant, ErrorKind, PiccoloError, Value};

use super::op::Opcode;

use std::collections::HashMap;

/// Interprets compiled Piccolo bytecode.
///
/// Contains a [`Chunk`] from which it executes instructions, a global variable hash
/// table, a stack for temporary values and local variables, and a [`Heap`] for long-lived
/// objects that require heap allocation, like strings, class instances, and others.
///
/// [`Chunk`]: ../chunk/struct.Chunk.html
/// [`Heap`]: ../memory/struct.Heap.html
pub struct Machine {
    ip: usize,
    globals: HashMap<String, Value>,
    stack: Vec<Value>,
    heap: Heap,
}

impl Default for Machine {
    fn default() -> Machine {
        Machine::new()
    }
}

impl Machine {
    /// Creates a new machine from a chunk.
    pub fn new() -> Self {
        Machine {
            ip: 0,
            globals: HashMap::new(),
            stack: Vec::new(),
            heap: Heap::new(1024),
        }
    }

    /// Get the [`Heap`] of a VM.
    ///
    /// [`Heap`]: ../memory/struct.Heap.html
    pub fn heap(&mut self) -> &mut Heap {
        &mut self.heap
    }

    // TODO: determine if self.ip - 1 is necessary
    // this method is only ever called after self.ip is incremented
    // theoretically a program should never start with Opcode::Pop
    fn pop(&mut self, chunk: &Chunk) -> Result<Value, PiccoloError> {
        self.stack.pop().ok_or_else(|| {
            PiccoloError::new(ErrorKind::StackUnderflow {
                op: chunk.data[self.ip - 1].into(),
            })
            .line(chunk.get_line_from_index(self.ip))
            .msg("file a bug report!")
        })
    }

    #[allow(dead_code)]
    fn peek_back(&self, dist: usize, chunk: &Chunk) -> Result<&Value, PiccoloError> {
        self.stack.get(self.stack.len() - dist - 1).ok_or_else(|| {
            PiccoloError::new(ErrorKind::StackUnderflow {
                op: chunk.data[self.ip - 1].into(),
            })
            .line(chunk.get_line_from_index(self.ip))
            .msg_string(format!("peek_back({})", dist))
        })
    }

    // get a constant from the chunk
    fn peek_constant<'a>(&self, chunk: &'a Chunk) -> &'a Constant {
        trace!("peek_constant");
        // Opcode::Constant takes a two-byte operand, meaning it's necessary
        // to decode the high and low bytes. the machine is little-endian with
        // constant addresses.
        chunk
            .constants
            .get(chunk.read_short(self.ip) as usize)
            .unwrap()
    }

    /// Interprets the machine's bytecode, returning a Constant.
    pub fn start_at(&mut self, chunk: &Chunk, start: usize) -> Result<Constant, PiccoloError> {
        self.ip = start;
        self.interpret(chunk)
    }

    // TODO: probably even move out the heap from the machine
    pub fn interpret(&mut self, chunk: &Chunk) -> Result<Constant, PiccoloError> {
        while self.ip < chunk.data.len() {
            debug!(
                " ┌─{}{}",
                if self.ip + 1 == chunk.data.len() {
                    "─vm─exit─ "
                } else {
                    " "
                },
                super::memory::dbg_list(&self.stack, &self.heap),
            );
            debug!(
                " └─{} {}",
                if self.ip + 1 == chunk.data.len() {
                    "───────── "
                } else {
                    " "
                },
                chunk.disassemble_instruction(self.ip)
            );

            let inst = chunk.data[self.ip];
            self.ip += 1;

            // boolean argument to enable/disable string concatenation
            macro_rules! bin_op {
                ($opcode:path, $op:tt) => {
                    bin_op!($opcode, $op, false)
                };
                ($opcode:path, $op:tt, $concat:tt) => {
                    let rhs = self.pop(chunk)?;
                    let lhs = self.pop(chunk)?;
                    if lhs.is_double() {
                        let lhs = lhs.into::<f64>();
                        if rhs.is_double() {
                            let rhs = rhs.into::<f64>();
                            self.stack.push(Value::Double(lhs $op rhs));
                        } else if rhs.is_integer() {
                            let rhs = rhs.into::<i64>();
                            self.stack.push(Value::Double(lhs $op rhs as f64));
                        } else {
                            return Err(PiccoloError::new(ErrorKind::IncorrectType {
                                exp: "integer or double".into(),
                                got: format!("double {} {}", stringify!($op), self.heap.type_name(&rhs)),
                                op: $opcode,
                            })
                            .line(chunk.get_line_from_index(self.ip)));
                        }
                    } else if lhs.is_integer() {
                        let lhs = lhs.into::<i64>();
                        if rhs.is_integer() {
                            let rhs = rhs.into::<i64>();
                            self.stack.push(Value::Integer(lhs $op rhs));
                        } else if rhs.is_double() {
                            let rhs = rhs.into::<f64>();
                            self.stack.push(Value::Double(lhs as f64 $op rhs));
                        } else {
                            return Err(PiccoloError::new(ErrorKind::IncorrectType {
                                exp: "integer or double".into(),
                                got: format!("integer {} {}", stringify!($op), self.heap.type_name(&rhs)),
                                op: $opcode,
                            })
                            .line(chunk.get_line_from_index(self.ip)));
                        }
                    } else if $concat && self.heap.is_string(&lhs) {
                        let value = format!("{}{}", self.heap.fmt(&lhs), self.heap.fmt(&rhs));
                        let ptr = self.heap.alloc(Box::new(value));
                        self.stack.push(Value::Object(ptr));
                    } else {
                        return Err(PiccoloError::new(ErrorKind::IncorrectType {
                            exp: "integer or double".into(),
                            got: format!("{} {} {}", self.heap.type_name(&lhs), stringify!($op), self.heap.type_name(&rhs)),
                            op: $opcode,
                        })
                        .line(chunk.get_line_from_index(self.ip)));
                    }
                };
            }

            let op = inst.into();
            match op {
                Opcode::Pop => {
                    if self.ip == chunk.data.len() {
                        let value = self.pop(chunk)?;
                        return Ok(self.heap.value_into_constant(value));
                    }
                    self.pop(chunk)?;
                }
                Opcode::Return => {
                    let v = self.pop(chunk)?;
                    println!("{}", self.heap.fmt(&v));
                }
                Opcode::Constant => {
                    let v = self
                        .heap
                        .constant_into_value(self.peek_constant(chunk).clone());
                    self.stack.push(v);
                    self.ip += 2;
                }
                Opcode::Nil => self.stack.push(Value::Nil),
                Opcode::True => self.stack.push(Value::Bool(true)),
                Opcode::False => self.stack.push(Value::Bool(false)),

                Opcode::Negate => {
                    let v = self.pop(chunk)?;
                    if v.is_double() {
                        let v = v.into::<f64>();
                        self.stack.push(Value::Double(-v));
                    } else if v.is_integer() {
                        let v = v.into::<i64>();
                        self.stack.push(Value::Integer(-v));
                    } else {
                        return Err(PiccoloError::new(ErrorKind::IncorrectType {
                            exp: "integer or double".into(),
                            got: self.heap.type_name(&v).to_owned(),
                            op: Opcode::Negate,
                        })
                        .line(chunk.get_line_from_index(self.ip - 1)));
                    }
                }
                Opcode::Not => {
                    let v = self.pop(chunk)?;
                    if v.is_truthy() {
                        self.stack.push(Value::Bool(false));
                    } else {
                        self.stack.push(Value::Bool(true));
                    }
                }
                Opcode::Add => {
                    bin_op!(Opcode::Add, +, true);
                }
                Opcode::Subtract => {
                    bin_op!(Opcode::Subtract, -);
                }
                Opcode::Multiply => {
                    bin_op!(Opcode::Multiply, *);
                }
                Opcode::Divide => {
                    bin_op!(Opcode::Multiply, /);
                }
                Opcode::Modulo => {
                    bin_op!(Opcode::Multiply, %);
                }

                Opcode::Equal => {
                    let a = self.pop(chunk)?;
                    let b = self.pop(chunk)?;
                    self.stack
                        .push(Value::Bool(self.heap.eq(&a, &b).map_or_else(
                            || {
                                Err(PiccoloError::new(ErrorKind::IncorrectType {
                                    exp: self.heap.type_name(&a).to_owned(),
                                    got: self.heap.type_name(&b).to_owned(),
                                    op,
                                })
                                .line(chunk.get_line_from_index(self.ip - 1)))
                            },
                            Ok,
                        )?));
                }
                Opcode::Greater => {
                    let rhs = self.pop(chunk)?;
                    let lhs = self.pop(chunk)?;
                    if rhs.is_bool() || lhs.is_bool() {
                        return Err(PiccoloError::new(ErrorKind::IncorrectType {
                            exp: "that isn't bool".into(),
                            got: "bool".into(),
                            op,
                        })
                        .line(chunk.get_line_from_index(self.ip - 1)));
                    }
                    self.stack
                        .push(Value::Bool(self.heap.gt(&lhs, &rhs).map_or_else(
                            || {
                                Err(PiccoloError::new(ErrorKind::IncorrectType {
                                    exp: self.heap.type_name(&lhs).to_owned(),
                                    got: self.heap.type_name(&rhs).to_owned(),
                                    op,
                                })
                                .line(chunk.get_line_from_index(self.ip - 1)))
                            },
                            Ok,
                        )?));
                }
                Opcode::Less => {
                    let rhs = self.pop(chunk)?;
                    let lhs = self.pop(chunk)?;
                    if rhs.is_bool() || lhs.is_bool() {
                        return Err(PiccoloError::new(ErrorKind::IncorrectType {
                            exp: "that isn't bool".into(),
                            got: "bool".into(),
                            op,
                        })
                        .line(chunk.get_line_from_index(self.ip - 1)));
                    }
                    self.stack
                        .push(Value::Bool(self.heap.lt(&lhs, &rhs).map_or_else(
                            || {
                                Err(PiccoloError::new(ErrorKind::IncorrectType {
                                    exp: self.heap.type_name(&lhs).to_owned(),
                                    got: self.heap.type_name(&rhs).to_owned(),
                                    op,
                                })
                                .line(chunk.get_line_from_index(self.ip - 1)))
                            },
                            Ok,
                        )?));
                }
                Opcode::GreaterEqual => {
                    let rhs = self.pop(chunk)?;
                    let lhs = self.pop(chunk)?;
                    if rhs.is_bool() || lhs.is_bool() {
                        return Err(PiccoloError::new(ErrorKind::IncorrectType {
                            exp: "that isn't bool".into(),
                            got: "bool".into(),
                            op,
                        })
                        .line(chunk.get_line_from_index(self.ip - 1)));
                    }
                    self.stack
                        .push(Value::Bool(!self.heap.lt(&lhs, &rhs).map_or_else(
                            || {
                                Err(PiccoloError::new(ErrorKind::IncorrectType {
                                    exp: self.heap.type_name(&lhs).to_owned(),
                                    got: self.heap.type_name(&rhs).to_owned(),
                                    op,
                                })
                                .line(chunk.get_line_from_index(self.ip - 1)))
                            },
                            Ok,
                        )?));
                }
                Opcode::LessEqual => {
                    let rhs = self.pop(chunk)?;
                    let lhs = self.pop(chunk)?;
                    if rhs.is_bool() || lhs.is_bool() {
                        return Err(PiccoloError::new(ErrorKind::IncorrectType {
                            exp: "that isn't bool".into(),
                            got: "bool".into(),
                            op,
                        })
                        .line(chunk.get_line_from_index(self.ip - 1)));
                    }
                    self.stack
                        .push(Value::Bool(!self.heap.gt(&lhs, &rhs).map_or_else(
                            || {
                                Err(PiccoloError::new(ErrorKind::IncorrectType {
                                    exp: self.heap.type_name(&lhs).to_owned(),
                                    got: self.heap.type_name(&rhs).to_owned(),
                                    op,
                                })
                                .line(chunk.get_line_from_index(self.ip - 1)))
                            },
                            Ok,
                        )?));
                }

                Opcode::GetLocal => {
                    let slot = chunk.read_short(self.ip);
                    self.stack.push(self.stack[slot as usize]);
                    self.ip += 2;
                }
                Opcode::SetLocal => {
                    let slot = chunk.read_short(self.ip);
                    self.stack[slot as usize] = self.pop(chunk)?;
                    self.ip += 2;
                }
                Opcode::GetGlobal => {
                    let name = self.peek_constant(chunk).ref_string();
                    if let Some(var) = self.globals.get(name) {
                        self.stack.push(*var);
                    } else {
                        return Err(PiccoloError::new(ErrorKind::UndefinedVariable {
                            name: name.to_owned(),
                        })
                        .line(chunk.get_line_from_index(self.ip - 1)));
                    }
                    self.ip += 2;
                }
                Opcode::SetGlobal => {
                    if let Constant::String(name) = self.peek_constant(chunk) {
                        let name = name.clone();
                        let value = self.pop(chunk)?;
                        if self.globals.insert(name.clone(), value).is_none() {
                            return Err(PiccoloError::new(ErrorKind::UndefinedVariable { name })
                                .line(chunk.get_line_from_index(self.ip - 1)));
                        }
                        self.ip += 2;
                    }
                }
                Opcode::DeclareGlobal => {
                    if let Constant::String(name) = self.peek_constant(chunk) {
                        let name = name.clone();
                        let value = self.pop(chunk)?;
                        self.globals.insert(name, value);
                        self.ip += 2;
                    } else {
                        panic!("defined global with non-string name");
                    }
                }

                Opcode::Assert => {
                    let v = self.pop(chunk)?;
                    if !v.is_truthy() {
                        return Err(PiccoloError::new(ErrorKind::AssertFailed)
                            .line(chunk.get_line_from_index(self.ip - 1)));
                    }
                }
            }

            trace!("next instruction");
        }

        Ok(self
            .heap
            .value_into_constant(self.stack.pop().unwrap_or(Value::Nil)))
    }
}
