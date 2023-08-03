use std::{collections::HashMap, rc::Rc};

use crate::value::Value;

pub trait Environment {
    /// Search for a [Value] in the Environment.
    fn variable(&self, name: &str) -> Option<Rc<Value>>;

    /// Search for a [Function] in the Environment.
    fn function(&self, name: &str) -> Option<Rc<Function>>;
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
    variables: HashMap<String, Rc<Value>>,
    functions: HashMap<String, Rc<Function>>,
}

impl StaticEnvironment {
    /// Add or update a variable to the Environment.
    /// Note: All variable names are *case-insensitive*.
    pub fn add_var(&mut self, name: &str, value: Value) {
        let name = name.to_lowercase();
        let value = Rc::new(value);

        self.variables.insert(name, value);
    }

    /// Add or update a native function to the Environment.
    /// Note: All function names are *case-insensitive*.
    pub fn add_native_func(&mut self, name: &str, arity: usize, func: NativeFunction) {
        let name = name.to_lowercase();
        let value = Rc::new(Function { func, arity });

        self.functions.insert(name, value);
    }
}

impl Environment for StaticEnvironment {
    fn variable(&self, name: &str) -> Option<Rc<Value>> {
        self.variables.get(name).cloned()
    }

    fn function(&self, name: &str) -> Option<Rc<Function>> {
        self.functions.get(name).cloned()
    }
}
