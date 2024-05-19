//! Dynamic variables and function calls can be provided by an [`Environment`].

use std::{collections::HashMap, rc::Rc};

use crate::{
    stdlib::{NativeError, NativeFunction, NativeResult},
    value::Value,
};

/// An enum signaling if a matching function is provided by a [`ValidateEnvironment`].
pub enum FunctionResult<'a> {
    /// A matching function was found.
    Exists(&'a Function),
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
    ///
    /// # Errors
    ///
    /// Returns [`NativeError`] when encountering an error inside a [`NativeFunction`].
    fn call(&self, name: &str, params: &[Value]) -> NativeResult;
}

/// An environment used during **validation** of the [`Expression`](crate::Expression).
#[allow(clippy::module_name_repetitions)]
pub trait ValidateEnvironment {
    /// Checks if a variable with a matching name exists.
    fn variable_exists(&self, name: &str) -> bool;

    /// Checks if a function with a matching name and compatible arity exists.
    fn function_exists(&self, name: &str, arity: usize) -> FunctionResult;
}

/// The [Arity](https://en.wikipedia.org/wiki/Arity) of a [`NativeFunction`].
#[derive(Clone, Copy)]
pub enum Arity {
    Polyadic { required: usize, optional: usize },
    Variadic,
    None,
}

impl Arity {
    /// Declares an Arity with some required but no optional parameters.
    #[must_use]
    pub const fn required(required: usize) -> Self {
        Self::Polyadic {
            required,
            optional: 0,
        }
    }

    /// Declares an Arity with required and optional parameters.
    #[must_use]
    pub const fn optional(required: usize, optional: usize) -> Self {
        Self::Polyadic { required, optional }
    }
}

/// A wrapper to hold the [`NativeFunction`] and its arity.
#[derive(Clone)]
pub struct Function {
    pub name: String,
    pub func: NativeFunction,
    pub arity: Arity,
    pub params: String,
    pub pure: bool,
}

impl Function {
    /// Creates a new pure `Function` from  a declaration.
    /// Example: "max(left: Number, right: Number): Number")
    ///
    /// # Remarks
    ///
    /// If the declaration does not contain an opening brace, the whole string
    /// is used as name and the params are left empty.
    #[must_use]
    pub fn new(func: NativeFunction, arity: Arity, declaration: &str) -> Self {
        let (name, params) = parse_declaration(declaration);

        Self {
            name,
            func,
            arity,
            params,
            pure: true,
        }
    }

    /// Creates an impure `Function`.
    ///
    /// See also: [`Function::new`]
    pub fn impure(func: NativeFunction, arity: Arity, declaration: &str) -> Self {
        Self {
            pure: false,
            ..Self::new(func, arity, declaration)
        }
    }
}

fn parse_declaration(declaration: &str) -> (String, String) {
    declaration
        .split_once('(')
        .map(|(name, param)| (name.trim().to_string(), format!("({param}")))
        .unwrap_or((declaration.to_string(), String::new()))
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
}

impl ValidateEnvironment for StaticEnvironment {
    fn variable_exists(&self, name: &str) -> bool {
        self.variables.contains_key(&get_env_key(name))
    }

    fn function_exists(&self, name: &str, param_count: usize) -> FunctionResult {
        if let Some(function) = self.functions.get(&get_env_key(name)) {
            match function.arity {
                Arity::Polyadic { required, optional } => {
                    let lower = required;
                    let upper = required + optional;

                    if param_count < lower {
                        FunctionResult::WrongArity(param_count, lower)
                    } else if param_count > upper {
                        FunctionResult::WrongArity(param_count, upper)
                    } else {
                        FunctionResult::Exists(function)
                    }
                }
                Arity::Variadic if param_count > 0 => FunctionResult::Exists(function),
                Arity::Variadic => FunctionResult::WrongArity(param_count, 1), // variadic without parameters
                Arity::None => FunctionResult::WrongArity(param_count, 0),
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

    #[test]
    fn new_function() {
        fn test_func(_params: &[Value]) -> NativeResult {
            unreachable!()
        }

        let func = Function::new(test_func, Arity::None, "some_name(param: Number): Number");
        assert_eq!("some_name", func.name);
        assert_eq!("(param: Number): Number", func.params);

        let func = Function::new(test_func, Arity::None, "only_name");
        assert_eq!("only_name", func.name);
        assert_eq!("", func.params);
    }
}
