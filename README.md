# SLAC: Simple Logic & Arithmetic Compiler

SLAC is a small and simple compiler which converts a single expression statement into an [AST](https://en.wikipedia.org/wiki/Abstract_syntax_tree).

It is written in Rust and as such compiles easily as an executable, wasm module, or standalone DLL.

# Examples

## Library usage

```rust
use slac::{ast::Expression, compile};

fn main() {
    let ast = compile("1 * 2 + 3");

    let expected = Expression::Binary {
        left: Box::new(Expression::Binary {
            left: Box::new(Expression::Literal(Value::Number(1.0))),
            right: Box::new(Expression::Literal(Value::Number(2.0))),
            operator: Operator::Star,
        }),
        right: Box::new(Expression::Literal(Value::Number(3.0))),
        operator: Operator::Plus,
    };

    assert_eq!(result, Ok(expected));
}
```

## Interpreter

SLAC features a built-in [tree walk interpreter](https://en.wikipedia.org/wiki/Interpreter_(computing)#Abstract_syntax_tree_interpreters).
Create an `Environment` which houses the variables and user defined functions. Then use the `TreeWalkingInterpreter` class to execute the AST against the environment.

```rust
use slac::compile;
use slac::environment::Environment;
use slac::interpreter::TreeWalkingInterpreter;
use slac::value::Value;

fn main() {
    let ast = compile("some_var > 5").unwrap();
    let mut env = Environment::default();
    env.add_var("some_var", Value::Number(42.0));

    let result = TreeWalkingInterpreter::interprete(&env, &ast);

    assert_eq!(result, Value::Boolean(true));
}
```

## Script syntax

The script syntax itself is similar to Delphi Pascal code.

```pascal
// arithmetic operators
40 + 1 * 2
// > 42

// Integer Division and Modulo
50 div 20 mod 2
// > 2

// comparisons
50 + 50 = 100
// > True

// logical operators
True and not False
// > True

// grouping
(40 + 1) * 2
// > 82

// arrays
[1, 2, 3] + ['Four']
// > [1, 2, 3, 'Four']

// application defined external functions
someFunc(true)
// > depends on the definition of 'someFunc'

// application defined variables
SOME_VAR + -10
// > depends on the definition of 'SOME_VAR'

```

# Installation

The minimum required Rust toolchain version for SLAC is **1.70.0** or higher. 

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