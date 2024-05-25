//! Wrapper structs for native [`Function`]` definitions.

use crate::stdlib::NativeFunction;

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

#[cfg(test)]
mod test {
    use crate::{
        function::{Arity, Function},
        stdlib::NativeResult,
        Value,
    };

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
