//! Objects defined in Rust that may exist at runtime.

use crate::{ChunkIndex, Object, Value};

use crate::runtime::vm::Value2;
use core::fmt;

pub struct NativeFunction {
    pub(crate) arity: usize,
    pub(crate) name: String,
    pub(crate) function: fn(&[Value2]) -> Value2,
}

impl Object for NativeFunction {
    fn trace(&self) {}

    fn type_name(&self) -> &'static str {
        "native_fn"
    }
}

impl fmt::Debug for NativeFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<native fn {}>", self.name)
    }
}

impl fmt::Display for NativeFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Default, Clone, PartialEq)]
pub struct Function {
    arity: usize,
    name: String,
    chunk: ChunkIndex,
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<fn {}>", self.name)
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<fn {}>", self.name)
    }
}

impl Function {
    pub fn new(arity: usize, name: String, chunk: ChunkIndex) -> Self {
        Self { arity, name, chunk }
    }

    pub fn arity(&self) -> &usize {
        &self.arity
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn chunk(&self) -> ChunkIndex {
        self.chunk
    }
}

impl Object for Function {
    fn trace(&self) {
        // TODO: constants?
    }

    fn type_name(&self) -> &'static str {
        "fn"
    }
}

impl<T: Object> Object for std::cell::RefCell<T> {
    fn trace(&self) {
        self.borrow().trace();
    }
}

impl<T: Object> Object for Vec<T> {
    fn trace(&self) {
        for item in self {
            item.trace();
        }
    }

    fn type_name(&self) -> &'static str {
        "list"
    }
}

impl<T: Object> Object for &Vec<T> {
    fn trace(&self) {
        for item in *self {
            item.trace();
        }
    }

    fn type_name(&self) -> &'static str {
        "list"
    }
}

impl<K: std::fmt::Debug + Eq + std::hash::Hash, T: Object> Object for fnv::FnvHashMap<K, T> {
    fn trace(&self) {
        for value in self.values() {
            value.trace();
        }
    }

    fn type_name(&self) -> &'static str {
        "map"
    }
}

impl Object for bool {
    fn trace(&self) {}

    fn type_name(&self) -> &'static str {
        "bool"
    }
}

impl Object for i64 {
    fn trace(&self) {}

    fn type_name(&self) -> &'static str {
        "int"
    }
}

impl Object for f64 {
    fn trace(&self) {}

    fn type_name(&self) -> &'static str {
        "float"
    }
}

impl Object for String {
    fn trace(&self) {}

    fn type_name(&self) -> &'static str {
        "string"
    }
}
