use std::collections::HashMap;

use crate::value::Value;

pub trait Environment {
    /// Search for a [Value] in the Environment.
    fn variable(&self, name: &str) -> Option<&Value>;

    /// Search for a [Function] in the Environment.
    fn function(&self, name: &str) -> Option<&Function>;
}

/// A function pointer used by the [`Interpreter`](crate::interpreter::TreeWalkingInterpreter).
/// All parameters to the function are inside a single Vec<[`Value`]>.
pub type NativeFunction = fn(Vec<Value>) -> Result<Value, String>;

pub struct Function {
    pub func: NativeFunction,
    pub arity: usize,
}

#[derive(Default)]
pub struct StaticEnvironment {
    variables: HashMap<String, Value>,
    functions: HashMap<String, Function>,
}

impl StaticEnvironment {
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
}

impl Environment for StaticEnvironment {
    fn variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    fn function(&self, name: &str) -> Option<&Function> {
        self.functions.get(name)
    }
}
