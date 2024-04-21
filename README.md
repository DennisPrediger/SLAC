![Crates.io Version](https://img.shields.io/crates/v/slac)
![docs.rs](https://img.shields.io/docsrs/slac)
![GitHub License](https://img.shields.io/github/license/dennisprediger/slac)

# SLAC: The Simple Logic & Arithmetic Compiler

SLAC is a small and simple compiler which converts a single expression statement into an [AST](https://en.wikipedia.org/wiki/Abstract_syntax_tree). You can use SLAC to implement a [business rules engine](https://en.wikipedia.org/wiki/Business_rules_engine) isolated from you application code at runtime.

It is written in Rust, and as such compiles easily as an **executable, wasm module, or standalone DLL**.

# Examples

## Library usage

```rust
use slac::{compile, Expression, Operator, Value};

fn main() {
    let ast = compile("1 * 2 + 3");

    let expected = Expression::Binary {
        left: Box::new(Expression::Binary {
            left: Box::new(Expression::Literal {
                value: Value::Number(1.0),
            }),
            right: Box::new(Expression::Literal {
                value: Value::Number(2.0),
            }),
            operator: Operator::Multiply,
        }),
        right: Box::new(Expression::Literal {
            value: Value::Number(3.0),
        }),
        operator: Operator::Plus,
    };

    assert_eq!(ast, Ok(expected));
}
```

## Interpreter

SLAC features a built-in [tree walk interpreter](https://en.wikipedia.org/wiki/Interpreter_(computing)#Abstract_syntax_tree_interpreters).
Create an `Environment` which houses the variables and user defined functions. Then use the `TreeWalkingInterpreter` class to execute the AST against the environment. Optional use `add_stdlib` to add some common functions.

```rust
use slac::{compile, execute, stdlib::extend_environment, StaticEnvironment, Value};

fn main() {
    let ast = compile("max(some_var, 3) > 5").unwrap();
    let mut env = StaticEnvironment::default();
    extend_environment(&mut env);
    env.add_var("some_var", Value::Number(42.0));

    let result = execute(&env, &ast);

    assert_eq!(result, Some(Value::Boolean(true)));
}
```

## Script syntax

The script syntax itself is similar to Delphi Pascal code.

```pascal
// arithmetic operators
40 + 1 * 2 // = 42

// Integer Division and Modulo
50 div 20 mod 2 // = 2

// comparisons
50 + 50 = 100 // = True

// logical operators
True and not False // = True

// grouping
(40 + 1) * 2 // = 82

// arrays
[1, 2, 3] + ['Four'] // = [1, 2, 3, 'Four']

// application defined external functions
max(10, 20) // = 20

// application defined variables
pi * -10 // = -31,4
```

# Serialization

By using the `serde` **feature flag**, the `Expression` can be (de)serialized to various formats, most notably JSON. This can be useful to separate the compilation, validation and optimization in the backend from the execution in the frontend.

```rust
use slac::{compile, execute, Expression, optimize, StaticEnvironment, Value};

fn main() {
    let mut input = compile("50 * 3 > 149").unwrap();
    optimize(&mut input).unwrap();
    let json = serde_json::to_value(&input).unwrap();

    // = Store the JSON in a database and load it on the client

    let output = serde_json::from_value::<Expression>(json).unwrap();
    let env = StaticEnvironment::default();

    let result = execute(&env, &output).unwrap();

    assert_eq!(input, output);
    assert_eq!(result, Value::Boolean(true));
}
```

# Installation

The minimum required Rust toolchain version is **1.70.0**. 

Use `cargo add slac` to install the library from [crates.io](https://crates.io/crates/slac) as a dependency in your application.

# License

Copyright 2023 Dennis Prediger

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.