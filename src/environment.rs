use std::collections::HashMap;

use crate::value::Value;

type NativeFunction = fn(Vec<Value>) -> Result<Value, String>;

pub struct Function {
    func: NativeFunction,
    arity: usize,
}

pub struct Environment {
    variables: HashMap<String, Value>,
    functions: HashMap<String, Function>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn add_var(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn add_native_func(&mut self, name: String, arity: usize, func: NativeFunction) {
        self.functions.insert(name, Function { func, arity });
    }

    pub fn get_var(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    pub fn get_func(&self, name: &str, param_count: usize) -> Option<&NativeFunction> {
        self.functions
            .get(name)
            .filter(|f| f.arity == param_count)
            .map(|f| &f.func)
    }
}
