use std::{collections::HashMap, rc::Rc};

use crate::value::Value;

pub enum FunctionResult {
    Exists,
    NotFound,
    WrongArity,
}

pub trait Environment {
    /// Get a variable [`Value`] from the Environment.
    fn variable(&self, name: &str) -> Option<Rc<Value>>;

    /// Call a [`Function`] and return a [`Value`].
    fn call(&self, name: &str, params: &[Value]) -> Option<Value>;

    /// Check if a [`Function`] exists.
    fn function(&self, name: &str, arity: usize) -> FunctionResult;
}

/// A function pointer used to execute native Rust functions.
/// All parameters to the function are inside a single Vec<[`Value`]>.
pub type NativeFunction = fn(&[Value]) -> Result<Value, String>;

pub struct Function {
    pub func: NativeFunction,
    pub arity: Option<usize>,
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
    pub fn add_native_func(&mut self, name: &str, arity: Option<usize>, func: NativeFunction) {
        let name = name.to_lowercase();
        let value = Rc::new(Function { func, arity });

        self.functions.insert(name, value);
    }
}

impl Environment for StaticEnvironment {
    fn variable(&self, name: &str) -> Option<Rc<Value>> {
        self.variables.get(&name.to_lowercase()).cloned()
    }

    fn call(&self, name: &str, params: &[Value]) -> Option<Value> {
        let function = self.functions.get(&name.to_lowercase())?;
        let call = function.func;

        call(params).ok()
    }

    fn function(&self, name: &str, arity: usize) -> FunctionResult {
        match self.functions.get(&name.to_lowercase()) {
            Some(function) => {
                if function.arity.map_or(true, |a| a == arity) {
                    FunctionResult::Exists
                } else {
                    FunctionResult::WrongArity
                }
            }
            None => FunctionResult::NotFound,
        }
    }
}
