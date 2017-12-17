
use ::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Env {
    pub inner: Vec<HashMap<String, value::Value>>,
}

impl Env {
    pub fn new() -> Self {
        Env {
            inner: vec![HashMap::new()]
        }
    }

    pub fn with_parent(parent: Env) -> Self {
        let mut e = Env {
            inner: parent.inner
        };
        e.push();
        e
    }

    pub fn push(&mut self) {
        self.inner.push(HashMap::new());
    }

    pub fn push_parent(&mut self, mut parent: Env) {
        self.inner.append(&mut parent.inner);
    }

    pub fn pop(&mut self) -> Option<HashMap<String, value::Value>> {
        self.inner.pop()
    }

    pub fn define(&mut self, name: &str, value: value::Value) {
        for scope in self.inner.iter_mut().rev().skip(1) {
            if scope.contains_key(name) {
                scope.insert(name.to_owned(), value.clone());
                return
            }
        }

        self.inner.iter_mut().rev().nth(0)
            .map(|m| m.insert(name.to_owned(), value.clone()))
            .expect("env is empty");
    }

    pub fn get(&self, name: &str) -> Option<value::Value> {
        for scope in self.inner.iter().rev() {
            if scope.contains_key(name) {
                return scope.get(name).cloned()
            }
        }
        None
    }

    pub fn set_local(&mut self, name: &str, value: value::Value)  {
        self.inner.iter_mut().rev().nth(0)
            .map(|m| m.insert(name.to_owned(), value))
            .expect("env is empty");
    }

    // TODO: re-visit closures in ch. 11
    pub fn children(&self) -> Env {
        Env {
            inner: self.inner.iter().skip(1).cloned().collect()
        }
    }
}

impl std::fmt::Display for Env {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut s = String::from("env:\n");
        for (n, ctx) in self.inner.iter().rev().enumerate() {
            s.push_str(&format!("  layer {}\n", n));
            'inner: for (k, v) in ctx.iter() {
                s.push_str(&format!("    {} = ", k));
                match v {
                    &value::Value::Func(ref f) => {
                        s.push_str(&format!("fn {} ", f.name));
                        if f.is_native() {
                            s.push_str("(native)\n");
                            continue 'inner;
                        } else {
                            for arg in f.decl.as_ref().unwrap().args.iter() {
                                s.push_str(&format!("{}, ", arg.lexeme));
                            }
                            s.push_str("\n");
                            for stmt in f.decl.as_ref().unwrap().body.iter() {
                                s.push_str(&format!("      {}\n", AstPrinter.print_stmt(stmt)))
                            }
                        }
                    }
                    _ => s.push_str(&format!("{:?}\n", v))
                }
            }
        }
        write!(f, "{}", s)
    }
}

