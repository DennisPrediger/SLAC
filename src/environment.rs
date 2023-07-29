use std::collections::HashMap;

use crate::value::Value;

/// A function pointer used by the [`Interpreter`](crate::interpreter::TreeWalkingInterpreter).
/// All parameters to the function are inside a single Vec<[`Value`]>.
pub type NativeFunction = fn(Vec<Value>) -> Result<Value, String>;

pub struct Function {
    pub func: NativeFunction,
    pub arity: usize,
}

#[derive(Default)]
pub struct Environment {
    variables: HashMap<String, Value>,
    functions: HashMap<String, Function>,
}

impl Environment {
    /// Add or update a variable to the Environment.
    /// Note: All variable names are *case-insensitive*.
    pub fn add_var(&mut self, name: &str, value: Value) {
        let name = name.to_lowercase();
        self.variables.insert(name, value);
    }

    /// Add or update a native function to the Environment.
    /// Note: All function names are *case-insensitive*.
    pub fn add_native_func(&mut self, name: &str, arity: usize, func: NativeFunction) {
        let name = name.to_lowercase();
        self.functions.insert(name, Function { func, arity });
    }

    pub fn get_var(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    pub fn get_function(&self, name: &str) -> Option<&Function> {
        self.functions.get(name)
    }

    pub fn get_func(&self, name: &str, param_count: usize) -> Option<&NativeFunction> {
        self.get_function(name)
            .filter(|f| f.arity == param_count)
            .map(|f| &f.func)
    }
}
