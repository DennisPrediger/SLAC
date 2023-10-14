//! Dynamic variables and function calls can be provided by an [`Environment`].

use std::{collections::HashMap, rc::Rc};

use crate::{stdlib::NativeFunction, value::Value};

/// An enum signaling if a matching function is provided by a [`ValidateEnvironment`].
pub enum FunctionResult {
    /// A matching function was found.
    Exists,
    /// No function with was found matching the supplied name.
    NotFound,
    /// A function with a matching name, but an incompatible arity was found.
    WrongArity(usize, usize),
}

/// An environment used by the interpreter when executing an [`Expression`](crate::Expression).
/// It provides access to variables and native function calls.
pub trait Environment {
    /// Get a variable [`Value`] from the Environment.
    fn variable(&self, name: &str) -> Option<Rc<Value>>;

    /// Call a [`Function`] and may return a [`Value`].
    fn call(&self, name: &str, params: &[Value]) -> Option<Value>;
}

/// An environment used during **validation** of the [`Expression`](crate::Expression).
pub trait ValidateEnvironment {
    /// Checks if a variable with a matching name exists.
    fn variable_exists(&self, name: &str) -> bool;

    /// Checks if a function with a matchinbg name and compatible arity exists.
    fn function_exists(&self, name: &str, arity: usize) -> FunctionResult;
}

/// A wrapper to hold the [`NativeFunction`] and its arity.
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

/// Handle all variable and function names case-insensitive.
#[inline(always)]
fn get_env_key(name: &str) -> String {
    name.to_lowercase()
}

impl StaticEnvironment {
    /// Add or update a variable.
    pub fn add_variable(&mut self, name: &str, value: Value) {
        let key = get_env_key(name);
        let value = Rc::new(value);

        self.variables.insert(key, value);
    }

    /// Remove a variable and return its [`Rc<Value>`] if it existed.
    pub fn remove_variable(&mut self, name: &str) -> Option<Rc<Value>> {
        self.variables.remove(&get_env_key(name))
    }

    /// Add or update a [`NativeFunction`].
    pub fn add_function(
        &mut self,
        name: &str,
        arity: Option<usize>,
        optionals: usize,
        func: NativeFunction,
    ) {
        let key = get_env_key(name);
        let value = Rc::new(Function {
            func,
            arity,
            optionals,
        });

        self.functions.insert(key, value);
    }

    /// Remove a native function and return its [`Rc<Function>`] struct if it existed.
    pub fn remove_function(&mut self, name: &str) -> Option<Rc<Function>> {
        self.functions.remove(&get_env_key(name))
    }
}

impl Environment for StaticEnvironment {
    fn variable(&self, name: &str) -> Option<Rc<Value>> {
        self.variables.get(&get_env_key(name)).cloned()
    }

    fn call(&self, name: &str, params: &[Value]) -> Option<Value> {
        let function = self.functions.get(&get_env_key(name))?;
        let call = function.func;

        call(params).ok()
    }
}

impl ValidateEnvironment for StaticEnvironment {
    fn variable_exists(&self, name: &str) -> bool {
        self.variables.contains_key(&get_env_key(name))
    }

    fn function_exists(&self, name: &str, param_count: usize) -> FunctionResult {
        if let Some(function) = self.functions.get(&get_env_key(name)) {
            if let Some(arity) = function.arity {
                let lower = arity - function.optionals;
                let upper = arity;

                if param_count < lower {
                    FunctionResult::WrongArity(param_count, lower)
                } else if param_count > upper {
                    FunctionResult::WrongArity(param_count, upper)
                } else {
                    FunctionResult::Exists
                }
            } else {
                FunctionResult::Exists // variadic Function
            }
        } else {
            FunctionResult::NotFound
        }
    }
}
