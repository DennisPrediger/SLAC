//! Dynamic variables and function calls can be provided by an [`Environment`].

use std::{collections::HashMap, rc::Rc};

use crate::{
    function::{Arity, Function},
    stdlib::{NativeError, NativeResult},
    value::Value,
};

/// An enum signaling if a matching function is provided by a [`Environment`].
pub enum FunctionResult {
    /// A matching function was found.
    Exists { pure: bool },
    /// No function with was found matching the supplied name.
    NotFound,
    /// A function with a matching name, but an incompatible arity was found.
    WrongArity { min: usize, max: usize },
}

/// An environment used by the interpreter when executing an [`Expression`](crate::Expression).
/// It provides access to variables and native function calls.
pub trait Environment {
    /// Get a variable [`Value`] from the Environment.
    fn variable(&self, name: &str) -> Option<Rc<Value>>;

    /// Call a [`Function`] and may return a [`Value`].
    ///
    /// # Errors
    ///
    /// Returns [`NativeError`] when encountering an error inside a [`NativeFunction`].
    fn call(&self, name: &str, params: &[Value]) -> NativeResult;

    /// Checks if a variable with a matching name exists.
    fn variable_exists(&self, name: &str) -> bool;

    /// Checks if a function with a matching name and compatible arity exists.
    fn function_exists(&self, name: &str, arity: usize) -> FunctionResult;
}

/// An [`Environment`] implementation in which all variables and functions are
/// known ahead of execution. All variable and function names treated as *case-insensitive*.
#[allow(clippy::module_name_repetitions)]
#[derive(Default)]
pub struct StaticEnvironment {
    variables: HashMap<String, Rc<Value>>,
    functions: HashMap<String, Rc<Function>>,
}

/// Transforms all variable and function names to lowercase for case-insensitive lookup.
fn get_env_key(name: &str) -> String {
    name.to_lowercase()
}

impl StaticEnvironment {
    /// Adds or updates a single variable.
    pub fn add_variable(&mut self, name: &str, value: Value) {
        self.variables.insert(get_env_key(name), Rc::new(value));
    }

    /// Removes a variable and return its [`Rc<Value>`] if it existed.
    pub fn remove_variable(&mut self, name: &str) -> Option<Rc<Value>> {
        self.variables.remove(&get_env_key(name))
    }

    /// Clears all variables.
    pub fn clear_variables(&mut self) {
        self.variables.clear();
    }

    /// Adds or updates a [`NativeFunction`].
    pub fn add_function(&mut self, func: Function) {
        self.functions
            .insert(get_env_key(&func.name), Rc::new(func));
    }

    /// Calls `add_function` for a `Vec<Function>`.
    pub fn add_functions(&mut self, functions: Vec<Function>) {
        for func in functions {
            self.add_function(func);
        }
    }

    /// Removes a [`NativeFunction`] and return its [`Function`] if it existed.
    pub fn remove_function(&mut self, name: &str) -> Option<Rc<Function>> {
        self.functions.remove(&get_env_key(name))
    }

    /// Output all currently registered [`Function`] structs as [`Rc`].
    #[must_use]
    pub fn list_functions(&self) -> Vec<Rc<Function>> {
        self.functions.values().cloned().collect()
    }
}

impl Environment for StaticEnvironment {
    fn variable(&self, name: &str) -> Option<Rc<Value>> {
        self.variables.get(&get_env_key(name)).cloned()
    }

    fn call(&self, name: &str, params: &[Value]) -> NativeResult {
        let function = self
            .functions
            .get(&get_env_key(name))
            .ok_or(NativeError::FunctionNotFound(name.to_string()))?;

        let call = function.func;
        call(params)
    }

    fn variable_exists(&self, name: &str) -> bool {
        self.variables.contains_key(&get_env_key(name))
    }

    fn function_exists(&self, name: &str, param_count: usize) -> FunctionResult {
        if let Some(function) = self.functions.get(&get_env_key(name)) {
            match function.arity {
                Arity::Polyadic { required, optional } => {
                    let min = required;
                    let max = required + optional;

                    if param_count < min || param_count > max {
                        FunctionResult::WrongArity { min, max }
                    } else {
                        FunctionResult::Exists {
                            pure: function.pure,
                        }
                    }
                }
                Arity::Variadic if param_count > 0 => FunctionResult::Exists {
                    pure: function.pure,
                },
                Arity::Variadic => FunctionResult::WrongArity { min: 1, max: 99 }, // variadic without parameters
                Arity::None => FunctionResult::WrongArity { min: 0, max: 0 },
            }
        } else {
            FunctionResult::NotFound
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::{compile, execute};

    #[test]
    fn static_variables() {
        let mut env = StaticEnvironment::default();

        env.add_variable("some_var", Value::Number(42.0));
        let ast = compile("some_var = 42").unwrap();
        assert_eq!(Ok(Value::Boolean(true)), execute(&env, &ast));

        env.remove_variable("some_var");
        assert_eq!(Ok(Value::Boolean(false)), execute(&env, &ast));

        env.add_variable("some_var", Value::Number(42.0));
        let ast = compile("some_var = 42").unwrap();
        assert_eq!(Ok(Value::Boolean(true)), execute(&env, &ast));

        env.clear_variables();
        assert_eq!(Ok(Value::Boolean(false)), execute(&env, &ast));
    }

    #[test]
    fn static_functions() {
        fn test_func(_params: &[Value]) -> NativeResult {
            unreachable!()
        }
        let mut env = StaticEnvironment::default();

        env.add_function(Function::new(test_func, Arity::Variadic, "test(...)"));

        let registered = env.list_functions();
        assert_eq!(1, registered.len());
        assert_eq!("test", registered.first().unwrap().name);
        let removed = env.remove_function("test").unwrap();

        assert_eq!(removed.name, registered.first().unwrap().name);
    }
}
