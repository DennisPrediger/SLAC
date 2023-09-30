//! Dynamic variables and function calls can be provided by an [`Environment`].

use std::{collections::HashMap, rc::Rc};

use crate::{stdlib::NativeFunction, value::Value};

/// An enum signaling if a compatible function is provided by a [`ValidateEnvironment`].
pub enum FunctionResult {
    /// A compatible function was found.
    Exists,
    /// No function with the given name was found.
    NotFound,
    /// A function with the same name but an incompatible arity was found.
    WrongArity(usize),
}

/// An environment used during the **excution** in the interpreter.
pub trait Environment {
    /// Get a variable [`Value`] from the Environment.
    fn variable(&self, name: &str) -> Option<Rc<Value>>;

    /// Call a [`Function`] and may return a [`Value`].
    fn call(&self, name: &str, params: &[Value]) -> Option<Value>;
}

/// An environment used during **validation** of the [`Expression`](crate::Expression).
/// Only checks for existance.
pub trait ValidateEnvironment {
    /// Checks if a variable with a given name exists.
    fn variable_exists(&self, name: &str) -> bool;

    /// Checks if a function with a given name and a compatible arity exists.
    fn function_exists(&self, name: &str, arity: usize) -> FunctionResult;
}

pub struct Function {
    pub func: NativeFunction,
    pub arity: Option<usize>,
    pub optionals: usize,
}

/// An [`Environment`] implementation in which all variables and functions are
/// known ahead of execution. All variable and function names treated as *case-insensitive*.
#[derive(Default)]
pub struct StaticEnvironment {
    variables: HashMap<String, Rc<Value>>,
    functions: HashMap<String, Rc<Function>>,
}

impl StaticEnvironment {
    /// Add or update a variable to the Environment.
    pub fn add_variable(&mut self, name: &str, value: Value) {
        let name = name.to_lowercase();
        let value = Rc::new(value);

        self.variables.insert(name, value);
    }

    /// Add or update a native function to the Environment.
    pub fn add_function(
        &mut self,
        name: &str,
        arity: Option<usize>,
        optionals: usize,
        func: NativeFunction,
    ) {
        let name = name.to_lowercase();
        let value = Rc::new(Function {
            func,
            arity,
            optionals,
        });

        self.functions.insert(name, value);
    }

    /// Remove a registered native function and return its [`Function`] struct.
    pub fn remove_function(&mut self, name: &str) -> Option<Rc<Function>> {
        self.functions.remove(&name.to_lowercase())
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
}

impl ValidateEnvironment for StaticEnvironment {
    fn variable_exists(&self, name: &str) -> bool {
        self.variables.contains_key(&name.to_lowercase())
    }

    fn function_exists(&self, name: &str, arity: usize) -> FunctionResult {
        match self.functions.get(&name.to_lowercase()) {
            Some(function) => {
                match function.arity {
                    Some(function_arity) => {
                        let range = function_arity..=function_arity + function.optionals;

                        if range.contains(&arity) {
                            FunctionResult::Exists
                        } else {
                            FunctionResult::WrongArity(*range.end())
                        }
                    }
                    None => FunctionResult::Exists, // variadic
                }
            }
            None => FunctionResult::NotFound,
        }
    }
}
